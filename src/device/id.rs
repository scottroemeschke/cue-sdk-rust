use crate::internal::try_c_char_ptr_to_str;
use failure::_core::str::Utf8Error;
use std::os::raw::c_char;

/// The error that can occur if the iCUE SDK were to give us a `c_char` array that couldn't be
/// parsed as valid Utf8.
#[derive(Debug, Clone, Fail, PartialEq)]
#[fail(display = "Invalid utf8 for the device id from ffi: {:?}", _0)]
pub struct DeviceIdFromFfiError(pub Utf8Error);

/// The immutable ID of the `CueDevice`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceId(pub String);

impl DeviceId {
    pub(crate) fn from_ffi(ffi_id: [c_char; 128usize]) -> Result<Self, DeviceIdFromFfiError> {
        try_c_char_ptr_to_str(ffi_id.as_ptr())
            .map_err(|e| DeviceIdFromFfiError(e))
            .map(
                |maybe_str| maybe_str.unwrap(), /* we created the pointer, so we can unwrap */
            )
            .map(|str| DeviceId(str.to_string()))
    }
}
