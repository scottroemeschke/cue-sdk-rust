use core::ffi::c_char;
use std::fmt;

use bitflags::bitflags;
use cue_sdk_sys as ffi;

// ---------------------------------------------------------------------------
// DeviceId
// ---------------------------------------------------------------------------

/// An opaque device identifier returned by the SDK.
///
/// This is a fixed-size byte array that can be cheaply round-tripped through
/// FFI without allocation.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(pub(crate) ffi::CorsairDeviceId);

impl DeviceId {
    /// View the raw bytes as a C string pointer.
    pub(crate) fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }

    /// Create from the FFI type (copy).
    pub(crate) fn from_ffi(raw: ffi::CorsairDeviceId) -> Self {
        Self(raw)
    }
}

impl fmt::Debug for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeviceId(\"{}\")", self)
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0.map(|c| c as u8);
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let s = std::str::from_utf8(&bytes[..len]).unwrap_or("<invalid utf8>");
        f.write_str(s)
    }
}

// ---------------------------------------------------------------------------
// DeviceType (bitflags)
// ---------------------------------------------------------------------------

bitflags! {
    /// Bitmask of device types used for filtering in [`Session::get_devices`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DeviceType: u32 {
        const UNKNOWN           = ffi::CorsairDeviceType_CDT_Unknown;
        const KEYBOARD          = ffi::CorsairDeviceType_CDT_Keyboard;
        const MOUSE             = ffi::CorsairDeviceType_CDT_Mouse;
        const MOUSEMAT          = ffi::CorsairDeviceType_CDT_Mousemat;
        const HEADSET           = ffi::CorsairDeviceType_CDT_Headset;
        const HEADSET_STAND     = ffi::CorsairDeviceType_CDT_HeadsetStand;
        const FAN_LED_CONTROLLER = ffi::CorsairDeviceType_CDT_FanLedController;
        const LED_CONTROLLER    = ffi::CorsairDeviceType_CDT_LedController;
        const MEMORY_MODULE     = ffi::CorsairDeviceType_CDT_MemoryModule;
        const COOLER            = ffi::CorsairDeviceType_CDT_Cooler;
        const MOTHERBOARD       = ffi::CorsairDeviceType_CDT_Motherboard;
        const GRAPHICS_CARD     = ffi::CorsairDeviceType_CDT_GraphicsCard;
        const TOUCHBAR          = ffi::CorsairDeviceType_CDT_Touchbar;
        const GAME_CONTROLLER   = ffi::CorsairDeviceType_CDT_GameController;
        const ALL               = ffi::CorsairDeviceType_CDT_All;
    }
}

// ---------------------------------------------------------------------------
// DeviceInfo
// ---------------------------------------------------------------------------

/// Information about a connected Corsair device.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// The device type bitmask.
    pub device_type: DeviceType,
    /// Unique device identifier.
    pub id: DeviceId,
    /// Serial number string.
    pub serial: String,
    /// Model name string.
    pub model: String,
    /// Number of LEDs on this device.
    pub led_count: i32,
    /// Number of channels (for DIY devices).
    pub channel_count: i32,
}

impl DeviceInfo {
    /// Convert from the FFI struct.
    pub(crate) fn from_ffi(raw: &ffi::CorsairDeviceInfo) -> Self {
        Self {
            device_type: DeviceType::from_bits_truncate(raw.type_),
            id: DeviceId::from_ffi(raw.id),
            serial: c_char_array_to_string(&raw.serial),
            model: c_char_array_to_string(&raw.model),
            led_count: raw.ledCount,
            channel_count: raw.channelCount,
        }
    }
}

/// Convert a fixed-size `c_char` array to an owned `String`, stopping at the
/// first null byte.
fn c_char_array_to_string(arr: &[c_char]) -> String {
    let bytes: Vec<u8> = arr
        .iter()
        .map(|&c| c as u8)
        .take_while(|&b| b != 0)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}
