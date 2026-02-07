# AGENTS.md — cue-sdk-rust

## Project Overview

`cue-sdk` is a safe, high-level Rust wrapper for the Corsair iCUE SDK v4.
It depends on `cue-sdk-sys` (FFI bindings) which lives in a sibling directory
at `../cue-sdk-sys/cue-sdk-sys`.

## Architecture

- **Entry point**: `Session` (in `src/session.rs`). All SDK operations are
  methods on `Session`. Created via `cue_sdk::connect()`.
- **Drop-based cleanup**: `Session` calls `CorsairDisconnect` on drop;
  `EventSubscription` auto-unsubscribes on drop.
- **Callback safety**: `Pin<Box<mpsc::Sender>>` with `extern "C"` trampolines
  in `src/callback.rs`. All `unsafe` blocks must have `// SAFETY` comments.
- **Zero-copy LEDs**: `LedColor` is `#[repr(C)]` layout-identical to
  `CorsairLedColor`.
- **Modules**: `session`, `device`, `led`, `event`, `property`, `error`,
  `callback` (pub(crate)).

## Building

```sh
cargo check           # Needs cue-sdk-sys at ../cue-sdk-sys/cue-sdk-sys
cargo clippy -- -D warnings
cargo fmt --check
```

This crate wraps a native SDK and cannot run tests without iCUE hardware.
`cargo check` and `clippy` are the primary CI gates.

## Conventions

- Edition 2021, Rust stable.
- `thiserror` 2 for error types, `bitflags` 2 for flag types.
- One public error type: `SdkError`. Use `crate::error::check()` to convert
  FFI return codes.
- Device operations take `&DeviceId`, not indices.
- Every `unsafe` block must have a `// SAFETY:` comment explaining the
  invariants.
- Keep the module structure flat — avoid deep nesting.

## Features

- `async` — adds optional `tokio` dependency (only `sync` feature) for async
  event support.

## Examples

Located in `examples/`. Run with `cargo run --example <name>`.
Available: `connect`, `devices`, `set_colors`, `events`.
