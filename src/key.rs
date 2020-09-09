//! Contains the `KeyId` enum, with all of the key identifiers
//! that are supported from the iCUE SDK.
use cue_sdk_sys as ffi;

/// All of the key identifiers that are expected and supported
/// from the iCUE SDK.
#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
#[repr(u32)]
pub enum KeyId {
    KeyboardG1 = ffi::CorsairKeyId_CorsairKeyKb_G1,
    KeyboardG2 = ffi::CorsairKeyId_CorsairKeyKb_G2,
    KeyboardG3 = ffi::CorsairKeyId_CorsairKeyKb_G3,
    KeyboardG4 = ffi::CorsairKeyId_CorsairKeyKb_G4,
    KeyboardG5 = ffi::CorsairKeyId_CorsairKeyKb_G5,
    KeyboardG6 = ffi::CorsairKeyId_CorsairKeyKb_G6,
    KeyboardG7 = ffi::CorsairKeyId_CorsairKeyKb_G7,
    KeyboardG8 = ffi::CorsairKeyId_CorsairKeyKb_G8,
    KeyboardG9 = ffi::CorsairKeyId_CorsairKeyKb_G9,
    KeyboardG10 = ffi::CorsairKeyId_CorsairKeyKb_G10,
    KeyboardG11 = ffi::CorsairKeyId_CorsairKeyKb_G11,
    KeyboardG12 = ffi::CorsairKeyId_CorsairKeyKb_G12,
    KeyboardG13 = ffi::CorsairKeyId_CorsairKeyKb_G13,
    KeyboardG14 = ffi::CorsairKeyId_CorsairKeyKb_G14,
    KeyboardG15 = ffi::CorsairKeyId_CorsairKeyKb_G15,
    KeyboardG16 = ffi::CorsairKeyId_CorsairKeyKb_G16,
    KeyboardG17 = ffi::CorsairKeyId_CorsairKeyKb_G17,
    KeyboardG18 = ffi::CorsairKeyId_CorsairKeyKb_G18,
    MouseM1 = ffi::CorsairKeyId_CorsairKeyMouse_M1,
    MouseM2 = ffi::CorsairKeyId_CorsairKeyMouse_M2,
    MouseM3 = ffi::CorsairKeyId_CorsairKeyMouse_M3,
    MouseM4 = ffi::CorsairKeyId_CorsairKeyMouse_M4,
    MouseM5 = ffi::CorsairKeyId_CorsairKeyMouse_M5,
    MouseM6 = ffi::CorsairKeyId_CorsairKeyMouse_M6,
    MouseM7 = ffi::CorsairKeyId_CorsairKeyMouse_M7,
    MouseM8 = ffi::CorsairKeyId_CorsairKeyMouse_M8,
    MouseM9 = ffi::CorsairKeyId_CorsairKeyMouse_M9,
    MouseM10 = ffi::CorsairKeyId_CorsairKeyMouse_M10,
    MouseM11 = ffi::CorsairKeyId_CorsairKeyMouse_M11,
    MouseM12 = ffi::CorsairKeyId_CorsairKeyMouse_M12,
}
