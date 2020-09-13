//! Top-level SDK functionality, including getting devices, writing and flushing color buffers,
//! and subscribing to events.
//!
//! This module contains the `CueSdkClient` which is the primary interface to top level iCUE SDK
//! functionality. It also contains the various top level errors that can happen when interacting
//! with the `CueSdkClient`.
//!
use cue_sdk_sys as ffi;
use failure::_core::ffi::c_void;
use num_traits::FromPrimitive;

use super::device::{
    CueDevice, CueDeviceFromDeviceInfoAndIndexError, CueDeviceInfo, CueDeviceInfoFromFfiError,
    DeviceCount,
};

use crate::errors::{get_last_error, CueSdkError, CueSdkErrorResult};

use crate::event::{CueEvent, CueEventFromFfiError};
use crate::initialization::{perform_handshake, HandshakeError, ProtocolDetails};
use crate::led::{CueLed, LedColor, LedId};

use crate::device::DeviceIndex;
use std::collections::HashMap;
use std::os::raw::c_char;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "async")]
use tokio::sync::mpsc;
#[cfg(feature = "async")]
use crate::event::EventSubscription;
#[cfg(feature = "async")]
use std::sync::Arc;

type CueErrorFfiCallback =
    unsafe extern "C" fn(ctx: *mut c_void, was_successful: bool, err: ffi::CorsairError);
type CueEventFfiCallback =
    unsafe extern "C" fn(context: *mut c_void, event: *const ffi::CorsairEvent);

const DEFAULT_SDK_CLIENT_PRIORITY: u8 = 128;

/// The LayerPriority for the `CueSdkClient`.
/// All clients to the SDK start at 128, and higher values = higher priority (or rather authority).
/// See the `set_layer_priority` method for more details.
pub type LayerPriority = u8;

/// The primary struct for interacting with top level iCUE SDK functionality.
///
/// The struct has some internal book-keeping around exclusive access, and event subscriptions,
/// and will clean those up when it is dropped.
///
/// It also houses the `ProtocolDetails` for the current session, and the current `LayerPriority`.
#[derive(Debug)]
pub struct CueSdkClient {
    has_exclusive_access: AtomicBool,
    is_subscribed_to_events: AtomicBool,
    protocol_details: ProtocolDetails,
    priority: LayerPriority,
}

/// The error that can be returned from the `get_all_devices` method.
#[derive(Debug, Clone, Fail, PartialEq)]
pub enum GetAllDevicesError {
    #[fail(display = "Failed to get the device count, error: {:?}", _0)]
    GetDeviceCountError(Option<CueSdkError>),
    #[fail(display = "Failed to get at least some devices, errors: {:?}", _0)]
    GetDeviceAtIndexErrors(Vec<GetDeviceAtIndexError>),
}

/// The error that can be returned from the `get_device_at` method.
#[derive(Debug, Clone, Fail, PartialEq)]
pub enum GetDeviceAtIndexError {
    #[fail(display = "Failed to get device info from ffi: {:?}", _0)]
    DeviceInfoFromFfiError(CueDeviceInfoFromFfiError),
    #[fail(display = "Failed to get at least some devices, errors: {:?}", _0)]
    CueDeviceFromDeviceInfoAndIndexError(CueDeviceFromDeviceInfoAndIndexError),
}

/// The error that can be returned from the `request_exclusive_access` method.
#[derive(Debug, Clone, Fail)]
pub enum RequestExclusiveAccessError {
    #[fail(display = "Failed to get exclusive access, error: {:?}", _0)]
    CueSdkError(Option<CueSdkError>),
    #[fail(display = "We already have exclusive access")]
    AlreadyHaveExclusiveAccessError,
}

/// The error that can be returned from the `release_exclusive_access` method.
#[derive(Debug, Clone, Fail)]
pub enum ReleaseExclusiveAccessError {
    #[fail(display = "Failed to release exclusive access, error: {:?}", _0)]
    CueSdkError(Option<CueSdkError>),
    #[fail(display = "We do not have exclusive access, so nothing to release.")]
    DoNotHaveExclusiveAccessError,
}

/// The error that can be returned from any of the event subscription methods.
#[derive(Debug, Clone, Fail)]
pub enum SubscribeForEventsError {
    #[fail(display = "Failed to release exclusive access, error: {:?}", _0)]
    CueSdkError(Option<CueSdkError>),
    #[fail(display = "The iCUE SDK only supports a single subscription, and there is already an active subscription.")]
    AlreadyHaveAnActiveEventSubscriptionError,
}

/// The error that can be returned from any of the event subscription methods.
#[derive(Debug, Clone, Fail)]
pub enum UnsubscribeFromEventsError {
    #[fail(display = "Failed to release exclusive access, error: {:?}", _0)]
    CueSdkError(Option<CueSdkError>),
    #[fail(display = "The iCUE SDK only supports a single subscription, and there is already an active subscription.")]
    NoActiveSubscriptionError,
}


impl CueSdkClient {
    pub(crate) fn initialize() -> Result<Self, HandshakeError> {
        perform_handshake().map(|pd| CueSdkClient {
            has_exclusive_access: AtomicBool::new(true),
            is_subscribed_to_events: AtomicBool::new(false),
            protocol_details: pd,
            priority: DEFAULT_SDK_CLIENT_PRIORITY,
        })
    }

    /// Get an immutable reference to the protocol details for the current iCUE SDK handshake.
    fn get_protocol_details(&self) -> &ProtocolDetails {
        &self.protocol_details
    }

    /// Get the current layer priority.
    fn get_layer_priority(&self) -> LayerPriority {
        self.priority
    }

    /// Get the current number of connected "iCue" devices.
    pub fn get_device_count(&self) -> Result<DeviceCount, Option<CueSdkError>> {
        let device_count_or_error = unsafe { ffi::CorsairGetDeviceCount() };
        if device_count_or_error < 0 {
            Err(get_last_error())
        } else {
            Ok(device_count_or_error as u32)
        }
    }

    /// Get the device located at the specified index.
    ///
    /// The index should be at max, one less than the number of devices (devices are zero indexed
    /// matching the index you get back from `getAllDevices`.
    pub fn get_device_at(&self, index: u32) -> Result<CueDevice, GetDeviceAtIndexError> {
        unsafe {
            let info_ptr = ffi::CorsairGetDeviceInfo(index as i32);
            let device_info = CueDeviceInfo::from_ffi(info_ptr)
                .map_err(|e| GetDeviceAtIndexError::DeviceInfoFromFfiError(e))?;

            let device = CueDevice::from_device_info_and_index(index, device_info)
                .map_err(|e| GetDeviceAtIndexError::CueDeviceFromDeviceInfoAndIndexError(e))?;

            Ok(device)
        }
    }

    /// Get all devices currently connected, with their full details including
    /// leds.
    ///
    /// If some device pointers come back invalid, this method will fail entirely,
    /// returning errors for each device pointer that didn't match some expected invariant .
    pub fn get_all_devices(&self) -> Result<Vec<CueDevice>, GetAllDevicesError> {
        let device_count = self
            .get_device_count()
            .map_err(|e| GetAllDevicesError::GetDeviceCountError(e))?;

        if device_count == 0 {
            return Ok(vec![]);
        }

        let mut errs = Vec::<GetDeviceAtIndexError>::with_capacity(device_count as usize);
        let mut devices = Vec::<CueDevice>::with_capacity(device_count as usize);

        for index in 0..device_count {
            match self.get_device_at(index) {
                Ok(d) => devices.push(d),
                Err(e) => errs.push(e),
            }
        }

        if errs.is_empty() {
            Ok(devices)
        } else {
            Err(GetAllDevicesError::GetDeviceAtIndexErrors(errs))
        }
    }

    /// Request exclusive access control of the SDK.
    ///
    /// This means other clients can't "do" anything, but can read data.
    ///
    /// If you do have exclusive access, it can be released manually with the
    /// `release_exclusive_access_control` method or it will happen automatically
    /// when the `CueSdkClient` is dropped.
    ///
    /// The "default" mode is non-exclusive.
    pub fn request_exclusive_access_control(&self) -> Result<(), RequestExclusiveAccessError> {
        if self.has_exclusive_access.load(Ordering::SeqCst) {
            return Err(RequestExclusiveAccessError::AlreadyHaveExclusiveAccessError);
        }
        match unsafe {
            ffi::CorsairRequestControl(ffi::CorsairAccessMode_CAM_ExclusiveLightingControl)
        } {
            true => {
                self.has_exclusive_access.store(true, Ordering::SeqCst);
                Ok(())
            }
            false => Err(RequestExclusiveAccessError::CueSdkError(get_last_error())),
        }
    }

    /// Release exclusive access control to the SDK.
    ///
    /// This means other clients *can* conflict with the messages you are sending
    /// to the connected devices.
    ///
    /// Non-exclusive access is the "default" mode.
    pub fn release_exclusive_access_control(&self) -> Result<(), ReleaseExclusiveAccessError> {
        if !self.has_exclusive_access.load(Ordering::SeqCst) {
            return Err(ReleaseExclusiveAccessError::DoNotHaveExclusiveAccessError);
        }
        let successfully_released = unsafe {
            ffi::CorsairReleaseControl(ffi::CorsairAccessMode_CAM_ExclusiveLightingControl)
        };
        if successfully_released {
            self.has_exclusive_access.store(true, Ordering::SeqCst);
            Ok(())
        } else {
            Err(ReleaseExclusiveAccessError::CueSdkError(get_last_error()))
        }
    }

    /// Set the layer priority for the client, (higher value is a higher priority).
    ///
    /// By default, all clients start with 128.
    ///
    /// The "priority" mechanism here is not about speed or queuing, but is about
    /// when two commands "conflict" which one "wins".
    pub fn set_layer_priority(&self, priority: LayerPriority) -> CueSdkErrorResult {
        let failed = unsafe { ffi::CorsairSetLayerPriority(priority as i32) };
        if failed {
            Err(get_last_error())
        } else {
            Ok(())
        }
    }

    /// Update the native iCUE SDK color buffer, to then eventually write to the connected devices.
    ///
    /// The iCUE SDK has it's own internal data structures for updating LED colors.
    /// Instead of writing directly to devices, it batches calls, and does other internal items.
    ///
    /// For this reason, you always need to write to a buffer (here, or on a `CueLed` struct itself)
    /// and then flush the buffer when you want those commands to take effect.
    pub fn update_leds_color_buffer(
        &self,
        leds: &mut [(&LedColor, &mut CueLed)],
    ) -> Result<(), UpdateLedsColorBufferError> {
        // This is real gross, and shows we likely need to save these C structs "internally".
        // For instance, ffi properties on CueLed that we can set them directly, instead of
        // using things like tuples and hashmaps for "book-keeping"
        let num_leds = leds.len();
        let updates = leds
            .iter_mut()
            .map(|(c, l)| {
                let ffi_color = ffi::CorsairLedColor {
                    ledId: l.id.into(),
                    r: c.red as i32,
                    g: c.green as i32,
                    b: c.blue as i32,
                };
                (c, l, ffi_color)
            })
            .fold(
                HashMap::<DeviceIndex, Vec<(&LedColor, &mut CueLed, ffi::CorsairLedColor)>>::with_capacity(num_leds),
                |mut map, (color, led, ffi_color)| {
                    let existing = map.get_mut(&led.device_index);
                    match existing {
                        Some(v) => {
                            v.push((color, led, ffi_color));
                            map
                        }
                        None => {
                            map.insert(led.device_index, vec![(color, led, ffi_color)]);
                            map
                        }
                    }
                },
            );

        let mut errs = Vec::with_capacity(updates.len());
        for (device_index, colors) in updates {
            let mut just_ffi_colors = colors
                .iter()
                .map(|(_, _, ffi_color)| *ffi_color)
                .collect::<Vec<ffi::CorsairLedColor>>();
            let was_successful = unsafe {
                ffi::CorsairSetLedsColorsBufferByDeviceIndex(
                    device_index as i32,
                    colors.len() as i32,
                    just_ffi_colors.as_mut_ptr(),
                )
            };
            if was_successful {
                colors.into_iter().for_each(|(color, led, _)| {
                    led.last_buffed_color = Some(*color);
                })
            }
            if !was_successful {
                errs.push(get_last_error());
            }
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(UpdateLedsColorBufferError(errs))
        }
    }

    /// Queues a flush of the iCUE SDK buffer, calling the passed in closure when the flush completes (successfully or not).
    ///
    /// After updating the color buffer, flushing it will send the led update commands to the specified
    /// `CueLed`s.
    ///
    /// This can take "some" time, and so there is a synchronous and asynchronous option.
    ///
    pub fn flush_led_colors_update_buffer<F>(&self, mut closure: F) -> CueSdkErrorResult
    where
        F: FnMut(CueSdkErrorResult),
    {
        let mut wrapper_closure = |was_successful: bool, err: ffi::CorsairError| {
            if was_successful {
                closure(Ok(()));
            } else {
                closure(Err(CueSdkError::from_u32(err)))
            }
        };

        let cb = get_error_callback(&wrapper_closure);
        let immediate_result = unsafe {
            ffi::CorsairSetLedsColorsFlushBufferAsync(
                Some(cb),
                &mut wrapper_closure as *mut _ as *mut c_void,
            )
        };
        if !immediate_result {
            Err(get_last_error())
        } else {
            Ok(())
        }
    }

    /// Flushes the iCUE SDK buffer synchronously, blocking the current thread, and then returning the flush result.
    ///
    /// After updating the color buffer, flushing it will send the led update commands to the specified
    /// `CueLed`s.
    ///
    /// This can take "some" time, and so there is a synchronous and asynchronous option.
    pub fn flush_led_colors_update_buffer_sync(&self) -> CueSdkErrorResult {
        let was_successful = unsafe { ffi::CorsairSetLedsColorsFlushBuffer() };
        if was_successful {
            Ok(())
        } else {
            Err(get_last_error())
        }
    }

    /// Subscribe for various events emitted from the iCUE SDK, with the passed in closure.
    ///
    /// You can unsubscribe manually by calling `unsubscribe_from_events` or the `CueSdkClient`
    /// will unsubscribe automatically if it is subscribed at the time it is dropped.
    pub fn subscribe_for_events<F>(&self, mut closure: F) -> Result<(), SubscribeForEventsError>
    where
        F: FnMut(Result<CueEvent, CueEventFromFfiError>),
    {
        if self.is_subscribed_to_events.load(Ordering::SeqCst) {
            return Err(SubscribeForEventsError::AlreadyHaveAnActiveEventSubscriptionError);
        }

        let mut wrapper_closure =
            |ev: *const ffi::CorsairEvent| closure(CueEvent::from_ffi(unsafe { *ev }));

        let cb = get_event_callback(&wrapper_closure);
        let immediate_result = unsafe {
            ffi::CorsairSubscribeForEvents(Some(cb), &mut wrapper_closure as *mut _ as *mut c_void)
        };
        if !immediate_result {
            Err(SubscribeForEventsError::CueSdkError(get_last_error()))
        } else {
            self.is_subscribed_to_events.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    /// Subscribe for various events emitted from the iCUE SDK, returning an `EventSubscription`
    /// which contains async await event streaming methods.
    ///
    /// You can unsubscribe manually by calling `unsubscribe_from_events` or the `CueSdkClient`
    /// will unsubscribe automatically if it is subscribed at the time it is dropped.
    /// Additionally if the `EventSubscription` is dropped, or it's `unsubscribe` method is called
    /// you will be unsubscribed.
    #[cfg(feature = "async")]
    pub fn subscribe_for_events_async(&self) -> Result<EventSubscription, SubscribeForEventsError> {
        let (tx,rx) = mpsc::channel(20);
        let mut tx2 = Arc::new(Mutex::new(tx));
        if self.is_subscribed_to_events.load(Ordering::SeqCst) {
            return Err(SubscribeForEventsError::AlreadyHaveAnActiveEventSubscriptionError)
        };

        let mut wrapper_closure = {
            |ev: *const ffi::CorsairEvent| {
                let event = unsafe { *ev };
                // playing it "safe" as this code is called from C, and panicking there is UB
                if let Some(mut c) = tx2.lock()
                    .ok() {
                    c.send(CueEvent::from_ffi(event));
                }
            }
        };

        let cb = get_event_callback(&wrapper_closure);
        let immediate_result = unsafe {
            ffi::CorsairSubscribeForEvents(Some(cb), &mut wrapper_closure as *mut _ as *mut c_void)
        };
        if !immediate_result {
            Err(SubscribeForEventsError::CueSdkError(get_last_error()))
        } else {
            self.is_subscribed_to_events.store(true, Ordering::SeqCst);
            Ok(EventSubscription::new(rx, &self))
        }
    }

    /// Unsubscribe from all events.
    pub fn unsubscribe_from_events(&self) -> Result<(), UnsubscribeFromEventsError> {
        if self.is_subscribed_to_events.load(Ordering::SeqCst) {
            return Err(UnsubscribeFromEventsError::NoActiveSubscriptionError);
        }
        if unsafe { ffi::CorsairUnsubscribeFromEvents() } {
            self.is_subscribed_to_events.store(true, Ordering::SeqCst);
            Ok(())
        } else {
            Err(UnsubscribeFromEventsError::CueSdkError(get_last_error()))
        }
    }

    /// Returns the `LedID` for the provided key name, as a `c_char`.
    pub fn get_led_for_key_name(key_name: c_char) -> Result<LedId, Option<CueSdkError>> {
        let led_id = unsafe { ffi::CorsairGetLedIdForKeyName(key_name) };
        if led_id == ffi::CorsairLedId_CLI_Invalid {
            Err(get_last_error())
        } else {
            Ok(led_id.into())
        }
    }
}

/// The error that can be returned from the `update_leds_color_buffer` method.
#[derive(Debug, Clone, Fail, PartialEq)]
#[fail(
    display = "Failed to update the color buffer for at least some leds, errors: {:?}",
    _0
)]
pub struct UpdateLedsColorBufferError(Vec<Option<CueSdkError>>);

unsafe extern "C" fn error_callback<F>(
    ctx: *mut c_void,
    was_successful: bool,
    err: ffi::CorsairError,
) where
    F: FnMut(bool, ffi::CorsairError),
{
    let closure = &mut *(ctx as *mut F);
    closure(was_successful, err);
}

fn get_error_callback<F>(_closure: &F) -> CueErrorFfiCallback
where
    F: FnMut(bool, ffi::CorsairError),
{
    error_callback::<F>
}

unsafe extern "C" fn event_callback<F>(ctx: *mut c_void, ev: *const ffi::CorsairEvent)
where
    F: FnMut(*const ffi::CorsairEvent),
{
    let closure = &mut *(ctx as *mut F);
    closure(ev);
}

fn get_event_callback<F>(_closure: &F) -> CueEventFfiCallback
where
    F: FnMut(*const ffi::CorsairEvent),
{
    event_callback::<F>
}

/// When the CueSdkClient is dropped, we check for existing event subscriptions, or
/// exclusive access rights and release/unsubscribe if needed.
impl Drop for CueSdkClient {
    fn drop(&mut self) {
        if self.has_exclusive_access.load(Ordering::Acquire) {
            self.release_exclusive_access_control().unwrap_or(());
        }
        if self.is_subscribed_to_events.load(Ordering::Acquire) {
            self.unsubscribe_from_events().unwrap_or(());
        }
    }
}
