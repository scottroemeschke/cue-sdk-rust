use std::sync::mpsc;

use cue_sdk_sys as ffi;

use crate::callback;
use crate::device::DeviceId;
use crate::error::{self, Result};

// ---------------------------------------------------------------------------
// MacroKeyId
// ---------------------------------------------------------------------------

/// Identifier for a G/M/S macro key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MacroKeyId {
    Key1 = ffi::CorsairMacroKeyId_CMKI_1,
    Key2 = ffi::CorsairMacroKeyId_CMKI_2,
    Key3 = ffi::CorsairMacroKeyId_CMKI_3,
    Key4 = ffi::CorsairMacroKeyId_CMKI_4,
    Key5 = ffi::CorsairMacroKeyId_CMKI_5,
    Key6 = ffi::CorsairMacroKeyId_CMKI_6,
    Key7 = ffi::CorsairMacroKeyId_CMKI_7,
    Key8 = ffi::CorsairMacroKeyId_CMKI_8,
    Key9 = ffi::CorsairMacroKeyId_CMKI_9,
    Key10 = ffi::CorsairMacroKeyId_CMKI_10,
    Key11 = ffi::CorsairMacroKeyId_CMKI_11,
    Key12 = ffi::CorsairMacroKeyId_CMKI_12,
    Key13 = ffi::CorsairMacroKeyId_CMKI_13,
    Key14 = ffi::CorsairMacroKeyId_CMKI_14,
    Key15 = ffi::CorsairMacroKeyId_CMKI_15,
    Key16 = ffi::CorsairMacroKeyId_CMKI_16,
    Key17 = ffi::CorsairMacroKeyId_CMKI_17,
    Key18 = ffi::CorsairMacroKeyId_CMKI_18,
    Key19 = ffi::CorsairMacroKeyId_CMKI_19,
    Key20 = ffi::CorsairMacroKeyId_CMKI_20,
}

impl MacroKeyId {
    pub(crate) fn from_ffi(raw: ffi::CorsairMacroKeyId) -> Option<Self> {
        match raw {
            ffi::CorsairMacroKeyId_CMKI_1 => Some(Self::Key1),
            ffi::CorsairMacroKeyId_CMKI_2 => Some(Self::Key2),
            ffi::CorsairMacroKeyId_CMKI_3 => Some(Self::Key3),
            ffi::CorsairMacroKeyId_CMKI_4 => Some(Self::Key4),
            ffi::CorsairMacroKeyId_CMKI_5 => Some(Self::Key5),
            ffi::CorsairMacroKeyId_CMKI_6 => Some(Self::Key6),
            ffi::CorsairMacroKeyId_CMKI_7 => Some(Self::Key7),
            ffi::CorsairMacroKeyId_CMKI_8 => Some(Self::Key8),
            ffi::CorsairMacroKeyId_CMKI_9 => Some(Self::Key9),
            ffi::CorsairMacroKeyId_CMKI_10 => Some(Self::Key10),
            ffi::CorsairMacroKeyId_CMKI_11 => Some(Self::Key11),
            ffi::CorsairMacroKeyId_CMKI_12 => Some(Self::Key12),
            ffi::CorsairMacroKeyId_CMKI_13 => Some(Self::Key13),
            ffi::CorsairMacroKeyId_CMKI_14 => Some(Self::Key14),
            ffi::CorsairMacroKeyId_CMKI_15 => Some(Self::Key15),
            ffi::CorsairMacroKeyId_CMKI_16 => Some(Self::Key16),
            ffi::CorsairMacroKeyId_CMKI_17 => Some(Self::Key17),
            ffi::CorsairMacroKeyId_CMKI_18 => Some(Self::Key18),
            ffi::CorsairMacroKeyId_CMKI_19 => Some(Self::Key19),
            ffi::CorsairMacroKeyId_CMKI_20 => Some(Self::Key20),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

/// An event received from the iCUE SDK.
#[derive(Debug, Clone)]
pub enum Event {
    /// A device was connected or disconnected.
    DeviceConnectionChanged {
        device_id: DeviceId,
        is_connected: bool,
    },
    /// A macro key was pressed or released.
    KeyEvent {
        device_id: DeviceId,
        key_id: MacroKeyId,
        is_pressed: bool,
    },
}

impl Event {
    /// Parse a raw FFI event.  Returns `None` for unrecognised event IDs.
    pub(crate) fn from_ffi(raw: &ffi::CorsairEvent) -> Option<Self> {
        match raw.id {
            ffi::CorsairEventId_CEI_DeviceConnectionStatusChangedEvent => {
                // SAFETY: The event `id` field is `CEI_DeviceConnectionStatusChangedEvent`,
                // so the `deviceConnectionStatusChangedEvent` union variant is active
                // and the pointer is valid for the callback's lifetime.
                let inner = unsafe { &*raw.event_union.deviceConnectionStatusChangedEvent };
                Some(Event::DeviceConnectionChanged {
                    device_id: DeviceId::from_ffi(inner.deviceId),
                    is_connected: inner.isConnected,
                })
            }
            ffi::CorsairEventId_CEI_KeyEvent => {
                // SAFETY: The event `id` field is `CEI_KeyEvent`, so the `keyEvent`
                // union variant is active and the pointer is valid.
                let inner = unsafe { &*raw.event_union.keyEvent };
                let key_id = MacroKeyId::from_ffi(inner.keyId)?;
                Some(Event::KeyEvent {
                    device_id: DeviceId::from_ffi(inner.deviceId),
                    key_id,
                    is_pressed: inner.isPressed,
                })
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// EventSubscription
// ---------------------------------------------------------------------------

/// An active event subscription.  Events can be received via [`recv`](Self::recv)
/// or [`try_recv`](Self::try_recv).
///
/// When this value is dropped the subscription is automatically cancelled by
/// calling `CorsairUnsubscribeFromEvents`.
pub struct EventSubscription {
    rx: mpsc::Receiver<Event>,
    // Prevent the sender from being dropped while the SDK holds the pointer.
    _sender: callback::EventSender,
}

impl EventSubscription {
    /// Create a new subscription.  Called by `Session::subscribe_for_events`.
    pub(crate) fn new(sender: callback::EventSender, rx: mpsc::Receiver<Event>) -> Result<Self> {
        let ctx = callback::sender_as_context(&sender);
        // SAFETY: We pass a valid function pointer and a context pointer derived
        // from a pinned boxed sender that we keep alive in the returned struct.
        error::check(unsafe {
            ffi::CorsairSubscribeForEvents(Some(callback::event_trampoline), ctx)
        })?;
        Ok(Self {
            rx,
            _sender: sender,
        })
    }

    /// Block until the next event arrives.
    pub fn recv(&self) -> Option<Event> {
        self.rx.recv().ok()
    }

    /// Non-blocking receive.
    pub fn try_recv(&self) -> Option<Event> {
        self.rx.try_recv().ok()
    }

    /// Returns an iterator that blocks on each event.
    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        self.rx.iter()
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        // SAFETY: `CorsairUnsubscribeFromEvents` is safe to call at any time
        // and will stop the SDK from invoking the callback, after which the
        // pinned sender can be safely dropped.
        unsafe {
            let _ = ffi::CorsairUnsubscribeFromEvents();
        }
    }
}

// ---------------------------------------------------------------------------
// AsyncEventSubscription (feature = "async")
// ---------------------------------------------------------------------------

/// An active event subscription with an async receiver.
///
/// Events can be received via [`recv`](Self::recv).  When this value is
/// dropped the subscription is automatically cancelled by calling
/// `CorsairUnsubscribeFromEvents`.
///
/// Requires the `async` feature.
#[cfg(feature = "async")]
pub struct AsyncEventSubscription {
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    // Prevent the sender from being dropped while the SDK holds the pointer.
    _sender: callback::AsyncEventSender,
}

#[cfg(feature = "async")]
impl AsyncEventSubscription {
    /// Create a new async subscription.  Called by
    /// `Session::subscribe_for_events_async`.
    pub(crate) fn new(
        sender: callback::AsyncEventSender,
        rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    ) -> Result<Self> {
        let ctx = callback::async_sender_as_context(&sender);
        // SAFETY: We pass a valid function pointer and a context pointer
        // derived from a pinned boxed sender that we keep alive in the
        // returned struct.
        error::check(unsafe {
            ffi::CorsairSubscribeForEvents(Some(callback::async_event_trampoline), ctx)
        })?;
        Ok(Self {
            rx,
            _sender: sender,
        })
    }

    /// Await the next event from the SDK.
    ///
    /// Returns `None` if the sender is dropped (should not happen while the
    /// subscription is alive).
    pub async fn recv(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

#[cfg(feature = "async")]
impl Drop for AsyncEventSubscription {
    fn drop(&mut self) {
        // SAFETY: `CorsairUnsubscribeFromEvents` is safe to call at any time
        // and will stop the SDK from invoking the callback, after which the
        // pinned sender can be safely dropped.
        unsafe {
            let _ = ffi::CorsairUnsubscribeFromEvents();
        }
    }
}
