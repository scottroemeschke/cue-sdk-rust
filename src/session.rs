use std::mem::MaybeUninit;
use std::sync::mpsc;
use std::time::Duration;

use core::ffi::{c_char, c_int};
use cue_sdk_sys as ffi;

use crate::callback::{self, SessionStateChange};
use crate::device::{DeviceId, DeviceInfo, DeviceType};
use crate::error::{self, Result, SdkError};
#[cfg(feature = "async")]
use crate::event::AsyncEventSubscription;
use crate::event::{EventSubscription, MacroKeyId};
use crate::led::{LedColor, LedPosition};
use crate::property::{DataType, PropertyFlags, PropertyId, PropertyInfo, PropertyValue};
use std::ptr;

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

/// A semantic version triple as reported by the SDK.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

impl Version {
    pub(crate) fn from_ffi(v: &ffi::CorsairVersion) -> Self {
        Self {
            major: v.major,
            minor: v.minor,
            patch: v.patch,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// ---------------------------------------------------------------------------
// SessionDetails
// ---------------------------------------------------------------------------

/// Version information about the client, server, and host.
#[derive(Debug, Clone, Copy)]
pub struct SessionDetails {
    pub client_version: Version,
    pub server_version: Version,
    pub server_host_version: Version,
}

impl SessionDetails {
    pub(crate) fn from_ffi(d: &ffi::CorsairSessionDetails) -> Self {
        Self {
            client_version: Version::from_ffi(&d.clientVersion),
            server_version: Version::from_ffi(&d.serverVersion),
            server_host_version: Version::from_ffi(&d.serverHostVersion),
        }
    }
}

// ---------------------------------------------------------------------------
// SessionState
// ---------------------------------------------------------------------------

/// The current state of the SDK session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Invalid,
    Closed,
    Connecting,
    Timeout,
    ConnectionRefused,
    ConnectionLost,
    Connected,
    Unknown(u32),
}

impl SessionState {
    pub(crate) fn from_ffi(raw: ffi::CorsairSessionState) -> Self {
        match raw {
            ffi::CorsairSessionState_CSS_Invalid => Self::Invalid,
            ffi::CorsairSessionState_CSS_Closed => Self::Closed,
            ffi::CorsairSessionState_CSS_Connecting => Self::Connecting,
            ffi::CorsairSessionState_CSS_Timeout => Self::Timeout,
            ffi::CorsairSessionState_CSS_ConnectionRefused => Self::ConnectionRefused,
            ffi::CorsairSessionState_CSS_ConnectionLost => Self::ConnectionLost,
            ffi::CorsairSessionState_CSS_Connected => Self::Connected,
            other => Self::Unknown(other),
        }
    }
}

// ---------------------------------------------------------------------------
// AccessLevel
// ---------------------------------------------------------------------------

/// SDK access level for a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AccessLevel {
    Shared = ffi::CorsairAccessLevel_CAL_Shared,
    ExclusiveLightingControl = ffi::CorsairAccessLevel_CAL_ExclusiveLightingControl,
    ExclusiveKeyEventsListening = ffi::CorsairAccessLevel_CAL_ExclusiveKeyEventsListening,
    ExclusiveLightingControlAndKeyEventsListening =
        ffi::CorsairAccessLevel_CAL_ExclusiveLightingControlAndKeyEventsListening,
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// A connected session to the iCUE SDK.
///
/// All SDK operations are methods on this struct.  Dropping the session calls
/// `CorsairDisconnect`.
///
/// Only one `Session` should exist at a time per process.
pub struct Session {
    state_rx: mpsc::Receiver<SessionStateChange>,
}

// SAFETY: The iCUE SDK is documented as thread-safe.  All SDK functions may be
// called from any thread, and our callback trampolines only send through
// `mpsc::Sender` which is `Send`.
unsafe impl Send for Session {}
unsafe impl Sync for Session {}

impl Session {
    /// Initiate a connection to iCUE.
    ///
    /// This registers the session-state callback and calls `CorsairConnect`.
    /// Use [`wait_for_connection`](Self::wait_for_connection) afterwards to
    /// block until the session reaches the `Connected` state.
    pub fn connect() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        callback::install_session_sender(tx);

        // SAFETY: We pass a valid function pointer.  The context pointer is null
        // because the trampoline reads from the process-wide static instead of
        // dereferencing the context (see `session_state_trampoline`).
        error::check(unsafe {
            ffi::CorsairConnect(Some(callback::session_state_trampoline), ptr::null_mut())
        })?;

        Ok(Self { state_rx: rx })
    }

    /// Block until the session state becomes `Connected` or the timeout
    /// elapses.
    ///
    /// On success returns the [`SessionDetails`] that were provided with the
    /// `Connected` state change.
    ///
    /// Returns `Err(SdkError::NotConnected)` on timeout or if the session
    /// enters a terminal error state (refused, lost).
    pub fn wait_for_connection(&self, timeout: Duration) -> Result<SessionDetails> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Err(SdkError::NotConnected);
            }
            match self.state_rx.recv_timeout(remaining) {
                Ok(change) => {
                    let state = SessionState::from_ffi(change.state);
                    match state {
                        SessionState::Connected => {
                            return Ok(SessionDetails::from_ffi(&change.details));
                        }
                        SessionState::Connecting => continue,
                        _ => return Err(SdkError::NotConnected),
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => return Err(SdkError::NotConnected),
                Err(mpsc::RecvTimeoutError::Disconnected) => return Err(SdkError::NotConnected),
            }
        }
    }

    /// Get the current session details (client/server/host versions).
    pub fn details(&self) -> Result<SessionDetails> {
        let mut raw = MaybeUninit::<ffi::CorsairSessionDetails>::uninit();
        // SAFETY: We pass a valid pointer to uninitialised memory that the SDK
        // will write into.  On success, all fields are initialised.
        error::check(unsafe { ffi::CorsairGetSessionDetails(raw.as_mut_ptr()) })?;
        // SAFETY: `check` returned `Ok`, so the SDK has fully initialised `raw`.
        Ok(SessionDetails::from_ffi(unsafe { &raw.assume_init() }))
    }

    // ---- Devices ----------------------------------------------------------

    /// Enumerate connected devices matching the given type filter.
    pub fn get_devices(&self, filter: DeviceType) -> Result<Vec<DeviceInfo>> {
        let ffi_filter = ffi::CorsairDeviceFilter {
            deviceTypeMask: filter.bits() as c_int,
        };
        let mut buf = [MaybeUninit::<ffi::CorsairDeviceInfo>::uninit();
            ffi::CORSAIR_DEVICE_COUNT_MAX as usize];
        let mut count: c_int = 0;

        // SAFETY: `buf` is a stack-allocated array large enough for the SDK's
        // maximum device count.  `count` receives the actual number written.
        error::check(unsafe {
            ffi::CorsairGetDevices(
                &ffi_filter,
                buf.len() as c_int,
                buf.as_mut_ptr().cast(),
                &mut count,
            )
        })?;

        let devices = (0..count as usize)
            // SAFETY: The SDK has initialised exactly `count` elements.
            .map(|i| DeviceInfo::from_ffi(unsafe { buf[i].assume_init_ref() }))
            .collect();
        Ok(devices)
    }

    /// Get detailed information about a specific device.
    pub fn get_device_info(&self, device_id: &DeviceId) -> Result<DeviceInfo> {
        let mut raw = MaybeUninit::<ffi::CorsairDeviceInfo>::uninit();
        // SAFETY: `device_id.as_ptr()` is a valid null-terminated C string.
        // `raw` is valid uninitialised memory for the SDK to write into.
        error::check(unsafe { ffi::CorsairGetDeviceInfo(device_id.as_ptr(), raw.as_mut_ptr()) })?;
        // SAFETY: `check` returned `Ok`, so the SDK has fully initialised `raw`.
        Ok(DeviceInfo::from_ffi(unsafe { raw.assume_init_ref() }))
    }

    // ---- LEDs -------------------------------------------------------------

    /// Get the positions of all LEDs on a device.
    pub fn get_led_positions(&self, device_id: &DeviceId) -> Result<Vec<LedPosition>> {
        let mut buf = [MaybeUninit::<ffi::CorsairLedPosition>::uninit();
            ffi::CORSAIR_DEVICE_LEDCOUNT_MAX as usize];
        let mut count: c_int = 0;

        // SAFETY: `buf` is large enough for the maximum LED count.
        // `count` receives the actual number of positions written.
        error::check(unsafe {
            ffi::CorsairGetLedPositions(
                device_id.as_ptr(),
                buf.len() as c_int,
                buf.as_mut_ptr().cast(),
                &mut count,
            )
        })?;

        let positions = (0..count as usize)
            // SAFETY: The SDK has initialised exactly `count` elements.
            .map(|i| LedPosition::from_ffi(unsafe { buf[i].assume_init_ref() }))
            .collect();
        Ok(positions)
    }

    /// Set LED colors on a device immediately.
    ///
    /// `colors` must be a slice of [`LedColor`] with the LED LUIDs set
    /// correctly for the target device.
    pub fn set_led_colors(&self, device_id: &DeviceId, colors: &[LedColor]) -> Result<()> {
        // SAFETY: `LedColor` is `#[repr(C)]` and layout-identical to
        // `CorsairLedColor` (verified by compile-time assertions in led.rs),
        // so the pointer cast is valid.  `colors` is a valid slice.
        error::check(unsafe {
            ffi::CorsairSetLedColors(
                device_id.as_ptr(),
                colors.len() as c_int,
                colors.as_ptr().cast(),
            )
        })
    }

    /// Buffer LED colors for later flushing with
    /// [`flush_led_colors`](Self::flush_led_colors).
    pub fn set_led_colors_buffer(&self, device_id: &DeviceId, colors: &[LedColor]) -> Result<()> {
        // SAFETY: Same layout guarantee as `set_led_colors`.
        error::check(unsafe {
            ffi::CorsairSetLedColorsBuffer(
                device_id.as_ptr(),
                colors.len() as c_int,
                colors.as_ptr().cast(),
            )
        })
    }

    /// Flush all buffered LED color changes.
    ///
    /// This is a synchronous wrapper around `CorsairSetLedColorsFlushBufferAsync`:
    /// it blocks until the SDK signals completion.
    pub fn flush_led_colors(&self) -> Result<()> {
        let (sender, rx) = callback::flush_channel();
        let ctx = callback::sender_as_context(&sender);

        // SAFETY: We pass a valid trampoline and a context pointer to a pinned
        // sender.  `sender` stays alive on this stack frame until `rx.recv()`
        // returns, which happens after the SDK invokes the callback.
        error::check(unsafe {
            ffi::CorsairSetLedColorsFlushBufferAsync(Some(callback::flush_trampoline), ctx)
        })?;

        // Wait for the async callback to fire.
        match rx.recv() {
            Ok(code) => error::check(code),
            Err(_) => Err(SdkError::NotConnected),
        }
    }

    /// Read current LED colors from a device.
    ///
    /// The `colors` slice must have the `id` field of each element pre-set to
    /// the LED LUID to query; the SDK fills in the `r`, `g`, `b`, `a` values.
    pub fn get_led_colors(&self, device_id: &DeviceId, colors: &mut [LedColor]) -> Result<()> {
        // SAFETY: Same layout guarantee as `set_led_colors`.  The SDK reads
        // each element's `id` and writes the colour fields in place.
        error::check(unsafe {
            ffi::CorsairGetLedColors(
                device_id.as_ptr(),
                colors.len() as c_int,
                colors.as_mut_ptr().cast(),
            )
        })
    }

    /// Look up the LED LUID for a key name character on a keyboard device.
    pub fn get_led_luid_for_key_name(&self, device_id: &DeviceId, key_name: c_char) -> Result<u32> {
        let mut luid: ffi::CorsairLedLuid = 0;
        // SAFETY: `luid` is a valid output pointer.
        error::check(unsafe {
            ffi::CorsairGetLedLuidForKeyName(device_id.as_ptr(), key_name, &mut luid)
        })?;
        Ok(luid)
    }

    /// Set the layer priority for this client (0–255).
    pub fn set_layer_priority(&self, priority: u32) -> Result<()> {
        // SAFETY: No pointer arguments; pure value call.
        error::check(unsafe { ffi::CorsairSetLayerPriority(priority) })
    }

    // ---- Access control ---------------------------------------------------

    /// Request exclusive control of a device.
    pub fn request_control(&self, device_id: &DeviceId, level: AccessLevel) -> Result<()> {
        // SAFETY: `device_id.as_ptr()` is a valid null-terminated C string.
        error::check(unsafe {
            ffi::CorsairRequestControl(device_id.as_ptr(), level as ffi::CorsairAccessLevel)
        })
    }

    /// Release exclusive control of a device.
    pub fn release_control(&self, device_id: &DeviceId) -> Result<()> {
        // SAFETY: `device_id.as_ptr()` is a valid null-terminated C string.
        error::check(unsafe { ffi::CorsairReleaseControl(device_id.as_ptr()) })
    }

    // ---- Events -----------------------------------------------------------

    /// Subscribe to SDK events (device connect/disconnect, key events).
    ///
    /// Returns an [`EventSubscription`] which unsubscribes on drop.
    pub fn subscribe_for_events(&self) -> Result<EventSubscription> {
        let (sender, rx) = callback::event_channel();
        EventSubscription::new(sender, rx)
    }

    /// Subscribe to SDK events with an async receiver.
    ///
    /// Returns an [`AsyncEventSubscription`] whose [`recv`](AsyncEventSubscription::recv)
    /// method is `async`.  The subscription unsubscribes on drop.
    ///
    /// Requires the `async` feature.
    #[cfg(feature = "async")]
    pub fn subscribe_for_events_async(&self) -> Result<AsyncEventSubscription> {
        let (sender, rx) = callback::async_event_channel();
        AsyncEventSubscription::new(sender, rx)
    }

    /// Flush all buffered LED color changes asynchronously.
    ///
    /// This is the async counterpart to [`flush_led_colors`](Self::flush_led_colors):
    /// it `.await`s instead of blocking.
    ///
    /// Requires the `async` feature.
    #[cfg(feature = "async")]
    pub async fn flush_led_colors_async(&self) -> Result<()> {
        let (sender, mut rx) = callback::async_flush_channel();
        let ctx = callback::async_sender_as_context(&sender);

        // SAFETY: We pass a valid trampoline and a context pointer to a pinned
        // sender.  `sender` stays alive in this async fn's state until
        // `rx.recv().await` returns, which happens after the SDK invokes the
        // callback.
        error::check(unsafe {
            ffi::CorsairSetLedColorsFlushBufferAsync(Some(callback::async_flush_trampoline), ctx)
        })?;

        match rx.recv().await {
            Some(code) => error::check(code),
            None => Err(SdkError::NotConnected),
        }
    }

    /// Configure whether a macro key event should be intercepted.
    pub fn configure_key_event(
        &self,
        device_id: &DeviceId,
        key_id: MacroKeyId,
        is_intercepted: bool,
    ) -> Result<()> {
        let config = ffi::CorsairKeyEventConfiguration {
            keyId: key_id as ffi::CorsairMacroKeyId,
            isIntercepted: is_intercepted,
        };
        // SAFETY: `config` is a valid stack-allocated struct.
        error::check(unsafe { ffi::CorsairConfigureKeyEvent(device_id.as_ptr(), &config) })
    }

    // ---- Properties -------------------------------------------------------

    /// Get metadata about a device property.
    pub fn get_device_property_info(
        &self,
        device_id: &DeviceId,
        property: PropertyId,
        index: u32,
    ) -> Result<PropertyInfo> {
        let mut data_type: ffi::CorsairDataType = 0;
        let mut flags: u32 = 0;

        // SAFETY: Output pointers are valid stack-allocated values.
        error::check(unsafe {
            ffi::CorsairGetDevicePropertyInfo(
                device_id.as_ptr(),
                property.to_ffi(),
                index,
                &mut data_type,
                &mut flags,
            )
        })?;

        Ok(PropertyInfo {
            data_type: DataType::from_ffi(data_type).unwrap_or(DataType::Int32), // fallback for unknown types
            flags: PropertyFlags::from_bits_truncate(flags),
        })
    }

    /// Read a device property value.
    ///
    /// The SDK-allocated memory is freed immediately after the value is copied
    /// into an owned [`PropertyValue`].
    pub fn read_device_property(
        &self,
        device_id: &DeviceId,
        property: PropertyId,
        index: u32,
    ) -> Result<PropertyValue> {
        let mut prop = MaybeUninit::<ffi::CorsairProperty>::zeroed();

        // SAFETY: `prop` points to zeroed memory suitable for the SDK to write
        // into.  On success all fields are initialised.
        error::check(unsafe {
            ffi::CorsairReadDeviceProperty(
                device_id.as_ptr(),
                property.to_ffi(),
                index,
                prop.as_mut_ptr(),
            )
        })?;

        // SAFETY: `check` returned `Ok`, so the SDK has fully initialised `prop`.
        let mut prop = unsafe { prop.assume_init() };
        // SAFETY: The property was just initialised by the SDK and its `type_`
        // field matches the union variant.  `from_ffi_and_free` copies the data
        // out and calls `CorsairFreeProperty` to release SDK memory.
        unsafe { PropertyValue::from_ffi_and_free(&mut prop) }.ok_or(SdkError::InvalidOperation)
    }

    /// Write a boolean property to a device.
    pub fn write_device_property_bool(
        &self,
        device_id: &DeviceId,
        property: PropertyId,
        index: u32,
        value: bool,
    ) -> Result<()> {
        let prop = crate::property::make_bool_property(value);
        // SAFETY: `prop` is a valid stack-allocated struct with matching
        // `type_` and `value` fields.
        error::check(unsafe {
            ffi::CorsairWriteDeviceProperty(device_id.as_ptr(), property.to_ffi(), index, &prop)
        })
    }

    /// Write an integer property to a device.
    pub fn write_device_property_int32(
        &self,
        device_id: &DeviceId,
        property: PropertyId,
        index: u32,
        value: i32,
    ) -> Result<()> {
        let prop = crate::property::make_int32_property(value);
        // SAFETY: Same as `write_device_property_bool`.
        error::check(unsafe {
            ffi::CorsairWriteDeviceProperty(device_id.as_ptr(), property.to_ffi(), index, &prop)
        })
    }

    /// Write a float property to a device.
    pub fn write_device_property_float64(
        &self,
        device_id: &DeviceId,
        property: PropertyId,
        index: u32,
        value: f64,
    ) -> Result<()> {
        let prop = crate::property::make_float64_property(value);
        // SAFETY: Same as `write_device_property_bool`.
        error::check(unsafe {
            ffi::CorsairWriteDeviceProperty(device_id.as_ptr(), property.to_ffi(), index, &prop)
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Clear the static sender *first* so the SDK's background thread
        // cannot send into a half-dropped channel (fixes macOS SIGBUS, #18).
        callback::clear_session_sender();

        // SAFETY: `CorsairDisconnect` is safe to call at any time; it is a
        // no-op if not connected.  We ignore the return value because we
        // cannot propagate errors from `Drop`.
        unsafe {
            let _ = ffi::CorsairDisconnect();
        }
    }
}
