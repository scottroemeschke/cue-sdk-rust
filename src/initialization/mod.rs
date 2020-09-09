use cue_sdk_sys as ffi;
use failure::Fail;

use super::errors::{get_last_error, CueSdkError};

mod protocol_details;

pub use self::protocol_details::ProtocolDetails;
pub use self::protocol_details::ProtocolDetailsFromFfiError;

#[derive(Debug, Clone, Fail, PartialEq)]
pub enum HandshakeError {
    #[fail(display = "Initial handshake failed with error: {:?}", _0)]
    InitialHandshakeError(CueSdkError),
    #[fail(
        display = "Unexpected error on receiving protocol details from the CueSDK: {:?}",
        _0
    )]
    ProtocolDetailsError(ProtocolDetailsFromFfiError),
}

pub(super) fn perform_handshake() -> Result<ProtocolDetails, HandshakeError> {
    let ffi_protocol_details = unsafe { ffi::CorsairPerformProtocolHandshake() };
    match get_last_error() {
        Some(e) => Err(HandshakeError::InitialHandshakeError(e)),
        None => ProtocolDetails::from_ffi(ffi_protocol_details)
            .map_err(|e| HandshakeError::ProtocolDetailsError(e)),
    }
}
