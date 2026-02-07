# cue-sdk

[![Crates.io](https://img.shields.io/crates/v/cue-sdk)](https://crates.io/crates/cue-sdk)
[![docs.rs](https://img.shields.io/docsrs/cue-sdk)](https://docs.rs/cue-sdk)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A safe, high-level Rust wrapper for the [iCUE SDK v4](https://github.com/CorsairOfficial/cue-sdk).

For low-level (unsafe) FFI bindings, see the companion crate
[cue-sdk-sys](https://github.com/scottroemeschke/cue-sdk-sys).

## Prerequisites

You need the iCUE SDK native libraries. Download them from the
[iCUE SDK releases page](https://github.com/CorsairOfficial/cue-sdk/releases)
and set the environment variables required by
[cue-sdk-sys](https://crates.io/crates/cue-sdk-sys).

The SDK DLLs/dylibs must be available at runtime (e.g. in your executable's
directory or on your system `PATH`/`LD_LIBRARY_PATH`). If they are missing
you will get `STATUS_DLL_NOT_FOUND` on Windows.

iCUE must be running on the target machine for the SDK to connect.

## Quick Start

```rust
use std::time::Duration;
use cue_sdk::device::DeviceType;
use cue_sdk::led::LedColor;

let session = cue_sdk::connect().expect("connect failed");
session.wait_for_connection(Duration::from_secs(5)).expect("timeout");

let devices = session.get_devices(DeviceType::ALL).expect("get_devices");
for dev in &devices {
    println!("{} ({}, {} LEDs)", dev.model, dev.id, dev.led_count);
}
```

## Setting LED Colors

```rust
use std::time::Duration;
use cue_sdk::device::DeviceType;
use cue_sdk::led::LedColor;

let session = cue_sdk::connect().expect("connect failed");
session.wait_for_connection(Duration::from_secs(5)).expect("timeout");

let devices = session.get_devices(DeviceType::KEYBOARD).expect("get_devices");
let device = devices.first().expect("no keyboard found");

let positions = session.get_led_positions(&device.id).expect("get_led_positions");
let colors: Vec<LedColor> = positions
    .iter()
    .map(|pos| LedColor::rgb(pos.id, 255, 0, 0))
    .collect();

session.set_led_colors(&device.id, &colors).expect("set_led_colors");
```

## Listening for Events

```rust
use std::time::Duration;
use cue_sdk::event::Event;

let session = cue_sdk::connect().expect("connect failed");
session.wait_for_connection(Duration::from_secs(5)).expect("timeout");

let subscription = session.subscribe_for_events().expect("subscribe");
for event in subscription.iter() {
    match event {
        Event::DeviceConnectionChanged { device_id, is_connected } => {
            println!("Device {} {}", device_id,
                if is_connected { "connected" } else { "disconnected" });
        }
        Event::KeyEvent { device_id, key_id, is_pressed } => {
            println!("Key {:?} {} on {}", key_id,
                if is_pressed { "pressed" } else { "released" }, device_id);
        }
    }
}
```

## Async Event Listening

Enable the `async` feature to get `AsyncEventSubscription` and
`flush_led_colors_async()`:

```toml
[dependencies]
cue-sdk = { version = "0.1", features = ["async"] }
```

```rust
use std::time::Duration;
use cue_sdk::event::Event;

#[tokio::main]
async fn main() {
    let session = cue_sdk::connect().expect("connect failed");
    session.wait_for_connection(Duration::from_secs(5)).expect("timeout");

    let mut subscription = session.subscribe_for_events_async().expect("subscribe");
    while let Some(event) = subscription.recv().await {
        match event {
            Event::DeviceConnectionChanged { device_id, is_connected } => {
                println!("Device {} {}", device_id,
                    if is_connected { "connected" } else { "disconnected" });
            }
            Event::KeyEvent { device_id, key_id, is_pressed } => {
                println!("Key {:?} {} on {}", key_id,
                    if is_pressed { "pressed" } else { "released" }, device_id);
            }
        }
    }
}
```

## Features

| Feature | Description |
|---------|-------------|
| `async` | Adds `AsyncEventSubscription` and `flush_led_colors_async()` via optional `tokio` dependency |

## Examples

Run the included examples with:

```sh
cargo run --example connect        # Print SDK version info
cargo run --example devices        # List connected devices
cargo run --example set_colors     # Set all keyboard LEDs to red
cargo run --example events         # Listen for device/key events
cargo run --example events_async --features async  # Async event listener
```

## Architecture

- **`Session`** is the single entry point for all SDK operations. Call
  `cue_sdk::connect()` to create one; it calls `CorsairDisconnect` on drop.
- Devices are identified by **`DeviceId`** (a 128-byte string), not indices.
- **`LedColor`** is `#[repr(C)]` and layout-identical to the native
  `CorsairLedColor` struct for zero-copy FFI.
- **`EventSubscription`** auto-unsubscribes on drop.
- All `unsafe` blocks have `// SAFETY` comments.

## License

[MIT](LICENSE)
