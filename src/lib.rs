//!  cue-sdk - A safe wrapper for the Corsair iCUE SDK.
//!
//! cue-sdk provides core functionality for interfacing with the
//! Corsair iCUE SDK, through the [cue-sdk-sys](https://crates.io/crates/cue-sdk-sys) crate.
//!
//! # Quick Start
//!
//! Make sure you set the required environment variables for the [cue-sdk-sys](https://crates.io/crates/cue-sdk-sys)
//! dependency crate.
//!
//! If you need the binaries, the easiest place to get them is on the [Github Releases Page](https://github.com/CorsairOfficial/cue-sdk).
//! Since we can't build them from scratch (not open source) you have to get them yourself.
//!
//! This version of the crate is built against version [3.0.55](https://github.com/CorsairOfficial/cue-sdk/releases/tag/v3.0.355)
//! of the iCUE SDK.
//!
//! # Example Code
//!
//! ```
//! use cue_sdk::led::LedColor;
//! use cue_sdk::initialize;
//! let sdk = initialize()
//!     .expect("failed to initialize sdk");
//! let mut  devices = sdk.get_all_devices().expect("failed to get all devices");
//! let new_color = LedColor { red: 200, green: 20, blue: 165 };
//! for d in &mut devices {
//!     //print some info
//!     println!("Device: {:?} at index {:?} has led count: ${:?}",
//!         d.device_info.model, d.device_index, d.leds.len());
//!
//!     // set the first led in every device to our `new_color` color
//!     d.leds.first_mut().unwrap().update_color_buffer(new_color);
//! }
//! //flush the colors buffer (send to device hardware)
//! sdk.flush_led_colors_update_buffer_sync()
//!     .expect("failed to flush led buffer");
//! ```
//!
//! You can note from the following example, most "write" operations can fail
//! for a variety of reasons including but not limited to:
//!
//! - device state changes (devices have been unplugged/plugged in)
//! - ffi interfacing (pointer derefs, etc) fail due to undocumented breaking changes in the iCUE SDK,
//! or a bug in the crate code
//! - another client has requested exclusive access
//!
//! # Examples
//!
//! For additional examples see the [example code](https://github.com/scottroemeschke/cue-sdk-rust)
//! and run examples with `cargo run --example {example_name}`.
//!
//!
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate num_derive;

#[cfg(test)]
#[macro_use]
extern crate strum_macros;

pub(crate) mod internal;

pub mod device;
pub mod errors;
pub mod event;
pub mod initialization;
pub mod key;
pub mod led;
pub mod property;
pub mod sdk;

use crate::sdk::CueSdkClient;
use initialization::HandshakeError;

/// This is the single and only way currently to initialize the SDK.
///
/// This method can fail for a variety of reasons, the most likely being
/// that the Corsair iCUE software isn't running on the target machine.
///
/// See [`HandshakeError`] error, for more details.
///
/// We recommend you keep a single instance of the SDK for your application,
/// as the iCUE SDK does not expect multiple handshakes.
///
pub fn initialize() -> Result<CueSdkClient, HandshakeError> {
    CueSdkClient::initialize()
}
