//! Int32 and Boolean property structs, for reading various device properties for devices with
//! property lookup functionality.
use crate::errors::CueSdkError;

mod boolean;
mod int32;

#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(display = "Failed to refresh value, error: {:?}", _0)]
pub struct RefreshValueError(Option<CueSdkError>);

pub use self::boolean::{BooleanProperty, BooleanPropertyKey};
pub use self::int32::{Int32Property, Int32PropertyKey};
