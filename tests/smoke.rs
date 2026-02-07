//! Smoke test proving the FFI chain loads correctly without iCUE running.
//!
//! This test does NOT require iCUE hardware. It verifies that:
//! - The native SDK library loads
//! - FFI symbols resolve
//! - The callback trampoline fires
//! - Timeout logic works
//!
//! Without iCUE running, `wait_for_connection` should return `NotConnected`.

use std::time::Duration;

use cue_sdk::SdkError;

#[test]
fn connect_without_icue_returns_not_connected() {
    let session = cue_sdk::connect().expect("connect() should succeed even without iCUE");

    let result = session.wait_for_connection(Duration::from_millis(500));

    assert_eq!(
        result.unwrap_err(),
        SdkError::NotConnected,
        "expected NotConnected when iCUE is not running"
    );
}
