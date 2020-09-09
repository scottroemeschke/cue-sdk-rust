//! Contains the top level SDK error type, and `Result` type.
use cue_sdk_sys as ffi;
use num_traits::FromPrimitive;

/// SdkErrors received from the iCUE SDK directly.
#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u32)]
pub enum CueSdkError {
    ServerNotFound = ffi::CorsairError_CE_ServerNotFound,
    NoControl = ffi::CorsairError_CE_NoControl,
    ProtocolHandshakeMissing = ffi::CorsairError_CE_ProtocolHandshakeMissing,
    IncompatibleProtocol = ffi::CorsairError_CE_IncompatibleProtocol,
    InvalidArguments = ffi::CorsairError_CE_InvalidArguments,
}

pub(crate) fn get_last_error() -> Option<CueSdkError> {
    CueSdkError::from_u32(unsafe { cue_sdk_sys::CorsairGetLastError() })
}

/// A common `Result` type, defaulting the data to unit, and the error is an
/// `Option<CueSdkError>`.
///
/// It is an option because it is possible for the SDK to return a failure on a given operation,
/// but then when we check for the error, it says there is none, or (on a particularly bad day) it *could*
/// give a value that isn't expected or documented.
pub type CueSdkErrorResult<T = ()> = Result<T, Option<CueSdkError>>;
