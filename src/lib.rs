//! A safe, high-level wrapper for the Corsair iCUE SDK v4.
//!
//! # Quick Start
//!
//! ```no_run
//! use std::time::Duration;
//! use cue_sdk::device::DeviceType;
//! use cue_sdk::led::LedColor;
//!
//! let session = cue_sdk::connect().expect("connect failed");
//! session.wait_for_connection(Duration::from_secs(5)).expect("timeout");
//!
//! let devices = session.get_devices(DeviceType::ALL).expect("get_devices");
//! for dev in &devices {
//!     println!("{} ({})", dev.model, dev.id);
//! }
//! ```
//!
//! # Architecture
//!
//! [`Session`] is the single entry point for all SDK operations.  Call
//! [`connect()`] to create one; it calls `CorsairDisconnect` on drop.
//!
//! Device information is returned as plain data structs ([`DeviceInfo`],
//! [`LedPosition`]).  Operations that need a device take a [`&DeviceId`]
//! parameter.

pub(crate) mod callback;
pub mod device;
pub mod error;
pub mod event;
pub mod led;
pub mod property;
pub mod session;

pub use device::{DeviceId, DeviceInfo, DeviceType};
pub use error::{Result, SdkError};
pub use event::{Event, EventSubscription, MacroKeyId};
pub use led::{LedColor, LedPosition};
pub use property::{PropertyId, PropertyValue};
pub use session::{AccessLevel, Session, SessionDetails, SessionState, Version};

/// Connect to the iCUE SDK and return a [`Session`].
///
/// This is a convenience wrapper around [`Session::connect()`].
pub fn connect() -> Result<Session> {
    Session::connect()
}
