//! Contains the event types, and event errors that can be returned from the
//! iCUE SDK.
use cue_sdk_sys as ffi;
use num_traits::FromPrimitive;

use super::key::KeyId;
use crate::device::{DeviceId, DeviceIdFromFfiError};

/// The two-variant event that can come back from the iCUE SDK.
///
/// The first is a notification of device status connection changes, with
/// the device id and whether it is now connected or not.
///
/// The second is a notification of a key event (pressed or released). Note
/// you will only receive notifications of keys that are value `KeyId`s (primarily
/// corsair specific media and "G" keys).
#[derive(Debug, Clone, PartialEq)]
#[repr(u32)]
pub enum CueEvent {
    DeviceConnectedStatusChangedEvent(DeviceId, bool),
    KeyEvent(DeviceId, KeyId, bool),
}

/// All of the various reasons why creating a CueEvent from the FFI interface can fail.
#[derive(Debug, Clone, Fail, PartialEq)]
pub enum CueEventFromFfiError {
    #[fail(display = "Received non-utf8 device id, error: {:?}.", _0)]
    DeviceIdError(DeviceIdFromFfiError),
    #[fail(display = "Received unknown event type: {}.", _0)]
    UnknownEventType(u32),
    #[fail(display = "Received unknown key id: {}.", _0)]
    UnknownKeyId(u32),
    #[fail(display = "The deviceConnectionStatusChangedEvent pointer was null.")]
    NullPointerDeviceConnectStatusChangedEvent,
    #[fail(display = "The keyEvent pointer was null.")]
    NullPointerKeyEvent,
}

impl CueEvent {
    pub(crate) fn from_ffi(event: ffi::CorsairEvent) -> Result<CueEvent, CueEventFromFfiError> {
        match event.id {
            ffi::CorsairEventId_CEI_DeviceConnectionStatusChangedEvent => {
                if unsafe {
                    event
                        .event_union
                        .deviceConnectionStatusChangedEvent
                        .is_null()
                } {
                    return Err(CueEventFromFfiError::NullPointerDeviceConnectStatusChangedEvent);
                }
                let event = unsafe { *event.event_union.deviceConnectionStatusChangedEvent };
                let device_id = DeviceId::from_ffi(event.deviceId)
                    .map_err(|e| CueEventFromFfiError::DeviceIdError(e))?;
                Ok(CueEvent::DeviceConnectedStatusChangedEvent(
                    device_id,
                    event.isConnected,
                ))
            }
            ffi::CorsairEventId_CEI_KeyEvent => {
                if unsafe { event.event_union.keyEvent.is_null() } {
                    return Err(CueEventFromFfiError::NullPointerKeyEvent);
                }
                let event = unsafe { *event.event_union.keyEvent };
                let device_id = DeviceId::from_ffi(event.deviceId)
                    .map_err(|e| CueEventFromFfiError::DeviceIdError(e))?;
                let key_id = KeyId::from_u32(event.keyId);
                match key_id {
                    Some(k) => Ok(CueEvent::KeyEvent(device_id, k, event.isPressed)),
                    None => Err(CueEventFromFfiError::UnknownKeyId(event.keyId)),
                }
            }
            _ => Err(CueEventFromFfiError::UnknownEventType(event.id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use cue_sdk_sys as ffi;

    use std::ptr;

    use super::{CueEvent, CueEventFromFfiError};
    use crate::device::DeviceId;
    use crate::key::KeyId;

    const EXAMPLE_DEVICE_ID: &[i8; 128] = &[
        0x30, 0x40, 0x50, 0x20, 0x30, 0x20, 0x30, 0x40, 0x10, 0x11, 0x12, 0x13, 0x30, 0x40, 0x50,
        0x20, 0x30, 0x20, 0x30, 0x40, 0x10, 0x11, 0x12, 0x13, 0x30, 0x40, 0x50, 0x20, 0x30, 0x20,
        0x30, 0x40, 0x10, 0x11, 0x12, 0x13, 0x30, 0x40, 0x50, 0x20, 0x30, 0x20, 0x30, 0x40, 0x10,
        0x11, 0x12, 0x13, 0x30, 0x40, 0x50, 0x20, 0x30, 0x20, 0x30, 0x40, 0x10, 0x11, 0x12, 0x13,
        0x30, 0x40, 0x50, 0x20, 0x30, 0x20, 0x30, 0x40, 0x10, 0x11, 0x12, 0x13, 0x30, 0x40, 0x50,
        0x20, 0x30, 0x20, 0x30, 0x40, 0x10, 0x11, 0x12, 0x13, 0x30, 0x40, 0x50, 0x20, 0x30, 0x20,
        0x30, 0x40, 0x10, 0x11, 0x12, 0x13, 0x30, 0x40, 0x50, 0x20, 0x30, 0x20, 0x30, 0x40, 0x10,
        0x11, 0x12, 0x13, 0x30, 0x40, 0x50, 0x20, 0x30, 0x20, 0x30, 0x40, 0x10, 0x11, 0x12, 0x13,
        0x30, 0x40, 0x50, 0x20, 0x30, 0x20, 0x30, 0x40,
    ];

    #[test]
    fn from_ffi_with_unknown_event_type() {
        let unknown_event_type = 24;
        let ffi_value = ffi::CorsairEvent {
            id: unknown_event_type,
            event_union: ffi::CorsairEventUnion {
                deviceConnectionStatusChangedEvent: ptr::null(),
            },
        };
        assert_eq!(
            CueEvent::from_ffi(ffi_value).unwrap_err(),
            CueEventFromFfiError::UnknownEventType(unknown_event_type)
        );
    }

    #[test]
    fn from_ffi_device_connection_null_ptr() {
        let ffi_value = ffi::CorsairEvent {
            id: ffi::CorsairEventId_CEI_DeviceConnectionStatusChangedEvent,
            event_union: ffi::CorsairEventUnion {
                deviceConnectionStatusChangedEvent: ptr::null(),
            },
        };
        assert_eq!(
            CueEvent::from_ffi(ffi_value).unwrap_err(),
            CueEventFromFfiError::NullPointerDeviceConnectStatusChangedEvent
        );
    }

    #[test]
    fn from_ffi_device_connection_all_valid() {
        let event = ffi::CorsairDeviceConnectionStatusChangedEvent {
            deviceId: EXAMPLE_DEVICE_ID.clone(),
            isConnected: false,
        };
        let ffi_value = ffi::CorsairEvent {
            id: ffi::CorsairEventId_CEI_DeviceConnectionStatusChangedEvent,
            event_union: ffi::CorsairEventUnion {
                deviceConnectionStatusChangedEvent: &event
                    as *const ffi::CorsairDeviceConnectionStatusChangedEvent,
            },
        };
        assert_eq!(
            CueEvent::from_ffi(ffi_value).unwrap(),
            CueEvent::DeviceConnectedStatusChangedEvent(DeviceId("0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@".to_string()), false)
        )
    }

    #[test]
    fn from_ffi_key_press_null_ptr() {
        let ffi_value = ffi::CorsairEvent {
            id: ffi::CorsairEventId_CEI_KeyEvent,
            event_union: ffi::CorsairEventUnion {
                keyEvent: ptr::null(),
            },
        };
        assert_eq!(
            CueEvent::from_ffi(ffi_value).unwrap_err(),
            CueEventFromFfiError::NullPointerKeyEvent
        );
    }

    #[test]
    fn from_ffi_key_press_invalid_key_id() {
        let invalid_key_id = 1300;
        let event = ffi::CorsairKeyEvent {
            deviceId: EXAMPLE_DEVICE_ID.clone(),
            keyId: invalid_key_id,
            isPressed: true,
        };
        let ffi_value = ffi::CorsairEvent {
            id: ffi::CorsairEventId_CEI_KeyEvent,
            event_union: ffi::CorsairEventUnion {
                keyEvent: &event as *const ffi::CorsairKeyEvent,
            },
        };
        assert_eq!(
            CueEvent::from_ffi(ffi_value).unwrap_err(),
            CueEventFromFfiError::UnknownKeyId(invalid_key_id)
        )
    }

    #[test]
    fn from_ffi_key_press_all_valid() {
        let valid_key_id = ffi::CorsairKeyId_CorsairKeyKb_G2;
        let event = ffi::CorsairKeyEvent {
            deviceId: EXAMPLE_DEVICE_ID.clone(),
            keyId: valid_key_id,
            isPressed: true,
        };
        let ffi_value = ffi::CorsairEvent {
            id: ffi::CorsairEventId_CEI_KeyEvent,
            event_union: ffi::CorsairEventUnion {
                keyEvent: &event as *const ffi::CorsairKeyEvent,
            },
        };
        assert_eq!(
            CueEvent::from_ffi(ffi_value).unwrap(),
            CueEvent::KeyEvent(DeviceId("0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@\u{10}\u{11}\u{12}\u{13}0@P 0 0@".to_string()), KeyId::KeyboardG2, true)
        )
    }
}

#[cfg(feature = "async")]
use tokio::

struct EventSubscription {
    channel:
}