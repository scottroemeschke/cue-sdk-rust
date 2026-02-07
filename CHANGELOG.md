# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.1.1] - 2026-02-07

### Fixed
- Fix macOS SIGBUS on reconnect: session state callback now uses a process-wide static sender instead of a per-`Session` pinned pointer, preventing use-after-free when the SDK's background thread fires during teardown (#18).
- Fix CI smoke test deadlocks by running tests with `--test-threads=1` since the iCUE SDK uses global state (#19).

## [v0.1.0] - 2026-02-07

Complete rewrite for iCUE SDK v4 (backed by `cue-sdk-sys` 0.1.0).

### Changed
- **Session-based API**: `cue_sdk::connect()` → `Session` replaces the old `initialize()` → `CueSdkClient`.
- **Device IDs**: Devices identified by `DeviceId` (128-byte string newtype), not integer indices.
- **`DeviceType`**: Now uses `bitflags` (v4 bitmask) instead of a sequential enum.
- **`LedColor`**: `#[repr(C)]` layout-identical to `CorsairLedColor` with LUID + RGBA (replaces `LedId` + RGB).
- **Error handling**: Single `SdkError` type via `thiserror` 2, replacing `failure` + per-operation error types.
- **Event system**: Channel-based `EventSubscription` with auto-unsubscribe on `Drop`, replacing `AtomicBool` tracking.
- **Module structure**: Flat 7-file layout replaces 20+ files across 6 directories.
- **Edition**: Updated to Rust 2021.
- **Dependencies**: `thiserror` 2, `bitflags` 2, `tokio` 1.x (optional) — removed `failure`, `semver`, `num-derive`.

### Added
- `Session::wait_for_connection()` with timeout support.
- `Session::get_led_positions()`, `Session::get_led_colors()`, `Session::set_led_colors_buffer()`, `Session::flush_led_colors()`.
- `Session::request_control()` / `release_control()` for exclusive device access.
- `Session::configure_key_event()` for macro key interception.
- Device property read/write API (`read_device_property`, `write_device_property_bool/int32/float64`).
- `Session::set_layer_priority()`.
- New examples: `connect`, `set_colors`.

### Removed
- `CueSdkClient`, `CueDevice`, and all cached device state.
- `LedId` enum (~1000 lines) — replaced by LED LUIDs (`u32`) + keyboard constants.
- Old examples: `access_control`, `all_keys_red`, `async`, `device_properties`, `initialization`, `random_colors`.

## [v0.0.3]

### Changed
- Updates `cue_sdk_sys` to `0.0.5`.

### Fixed
- Fixed requiring all features of tokio, instead of just `sync`.
- Fixed async examples.
- Fixed various clippy warnings.

## [v0.0.2]

### Changed
- Uses version `0.0.4` of the `cue-sdk-sys` crate, which has `Send` and `Sync` for various
C structs coming from the iCUE SDK.

### Added
- Added `async` feature! Events and color buffer flushing can now be async/awaited.

## [v0.0.1]

Initial release for CUE SDK version 3.0.55.
