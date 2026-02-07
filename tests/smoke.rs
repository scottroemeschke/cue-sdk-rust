//! Smoke tests proving the FFI chain loads and the SDK responds to real calls.
//!
//! These tests do NOT require iCUE to be running. They verify that:
//! - The native SDK library loads and FFI symbols resolve
//! - `CorsairConnect` succeeds and the callback trampoline fires
//! - `CorsairGetSessionDetails` returns the client SDK version
//! - `CorsairGetDevices` fails gracefully when not connected
//! - `CorsairDisconnect` + re-`CorsairConnect` lifecycle works
//!
//! All checks are in a single test because the iCUE SDK uses global state
//! (only one Session per process) and Rust tests run in parallel by default.

use std::time::Duration;

use cue_sdk::{DeviceType, SdkError, Version};

#[test]
fn sdk_smoke_test() {
    // -- Step 1: Connect --
    // CorsairConnect starts an async connection attempt.  It should succeed
    // immediately even without iCUE; it only registers the callback and
    // begins trying to reach the server.
    eprintln!("[smoke] step 1: connect()");
    let session = cue_sdk::connect().expect("connect() should succeed even without iCUE");

    // -- Step 2: Read session details (real SDK data!) --
    // CorsairGetSessionDetails returns version info.  The *client* version is
    // baked into the native library, so it should be available regardless of
    // whether iCUE is running.
    eprintln!("[smoke] step 2: details()");
    let details = session
        .details()
        .expect("details() should succeed after connect()");

    // The client version must match the SDK we link against (v4.x.x).
    eprintln!("[smoke] client_version = {}", details.client_version);
    assert_eq!(
        details.client_version.major, 4,
        "expected client SDK major version 4, got {}",
        details.client_version
    );
    assert!(
        details.client_version.minor >= 0 && details.client_version.patch >= 0,
        "client version components should be non-negative: {}",
        details.client_version
    );

    // Without iCUE, server version should be zeroed.
    let zero = Version {
        major: 0,
        minor: 0,
        patch: 0,
    };
    assert_eq!(
        details.server_version, zero,
        "expected zeroed server version without iCUE, got {}",
        details.server_version
    );

    // -- Step 3: Wait for connection (expect timeout) --
    // The callback trampoline should fire with Connecting → Timeout states.
    eprintln!("[smoke] step 3: wait_for_connection()");
    let result = session.wait_for_connection(Duration::from_millis(500));
    assert_eq!(
        result.unwrap_err(),
        SdkError::NotConnected,
        "expected NotConnected when iCUE is not running"
    );

    // -- Step 4: Call get_devices (should fail gracefully) --
    // After the connection attempt timed out, device queries should return
    // an appropriate error rather than crashing or hanging.
    eprintln!("[smoke] step 4: get_devices()");
    let devices_result = session.get_devices(DeviceType::ALL);
    assert!(
        devices_result.is_err(),
        "get_devices should fail when not connected to iCUE"
    );

    // -- Step 5: Lifecycle — disconnect and reconnect --
    // Drop the first session (calls CorsairDisconnect), then create a new
    // one to verify the SDK can be re-initialized cleanly.
    eprintln!("[smoke] step 5: drop + reconnect");
    drop(session);

    let session2 = cue_sdk::connect().expect("reconnect after disconnect should succeed");
    eprintln!("[smoke] step 5b: details() on second session");
    let details2 = session2
        .details()
        .expect("details() should work on second session");
    assert_eq!(
        details2.client_version.major, 4,
        "client version should still be 4 after reconnect, got {}",
        details2.client_version
    );

    eprintln!("[smoke] all steps passed");
}
