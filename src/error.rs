use cue_sdk_sys as ffi;

/// All errors that can be returned by SDK operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SdkError {
    #[error("not connected to iCUE")]
    NotConnected,
    #[error("no control over the device")]
    NoControl,
    #[error("incompatible protocol version")]
    IncompatibleProtocol,
    #[error("invalid arguments")]
    InvalidArguments,
    #[error("invalid operation")]
    InvalidOperation,
    #[error("device not found")]
    DeviceNotFound,
    #[error("operation not allowed")]
    NotAllowed,
    #[error("unknown SDK error code: {0}")]
    Unknown(u32),
}

/// Convenience alias used throughout this crate.
pub type Result<T> = std::result::Result<T, SdkError>;

/// Convert a raw `CorsairError` code into a `Result<()>`.
pub(crate) fn check(code: ffi::CorsairError) -> Result<()> {
    match code {
        ffi::CorsairError_CE_Success => Ok(()),
        ffi::CorsairError_CE_NotConnected => Err(SdkError::NotConnected),
        ffi::CorsairError_CE_NoControl => Err(SdkError::NoControl),
        ffi::CorsairError_CE_IncompatibleProtocol => Err(SdkError::IncompatibleProtocol),
        ffi::CorsairError_CE_InvalidArguments => Err(SdkError::InvalidArguments),
        ffi::CorsairError_CE_InvalidOperation => Err(SdkError::InvalidOperation),
        ffi::CorsairError_CE_DeviceNotFound => Err(SdkError::DeviceNotFound),
        ffi::CorsairError_CE_NotAllowed => Err(SdkError::NotAllowed),
        other => Err(SdkError::Unknown(other)),
    }
}
