//! Contains the `CueLed` struct, and all of it's associated functionality and errors.
use cue_sdk_sys as ffi;

mod id;

use crate::device::DeviceIndex;
use crate::errors::{get_last_error, CueSdkError, CueSdkErrorResult};
use crate::internal::CuePropertyValueHolder;
use crate::led::CheckColorError::{LedColorError, SdkError};
use failure::_core::ops::Range;

pub use id::{
    CommanderLedId, DeviceLedId, GraphicsCardLedId, HeadsetLedId, HeadsetStandLedId,
    IntegratedLedId, KeyboardLedId, LedId, MotherboardLedId, MouseMatLedId,
};

/// This struct contains static information about the led (id, position, device index) as well
/// as the last color we wrote to the color buffer, and the last color that we manually checked
/// (confirmed read from device).
#[derive(Debug, Clone, PartialEq)]
pub struct CueLed {
    pub id: LedId,
    pub position: LedPosition,
    pub(crate) device_index: DeviceIndex,
    pub last_checked_color: Option<LedColor>,
    pub last_buffed_color: Option<LedColor>,
}

/// The two-variant enum of errors that can occur when checking the color of a `CueLed`.
#[derive(Debug, Clone, Fail, PartialEq)]
pub enum CheckColorError {
    #[fail(display = "Failed to check the color, error: {:?}.", _0)]
    SdkError(Option<CueSdkError>),
    #[fail(display = "Error creating valid LedColor, error: {:?}", _0)]
    LedColorError(LedColorFromFfiError),
}

impl CueLed {
    pub(crate) fn with_device_data_from_ffi(
        device_index: DeviceIndex,
        led: &ffi::CorsairLedPosition,
    ) -> Self {
        let id: LedId = led.ledId.into();
        let position = LedPosition {
            height: led.height,
            left: led.left,
            top: led.top,
            width: led.width,
        };
        CueLed {
            device_index,
            id,
            position,
            last_checked_color: None,
            last_buffed_color: None,
        }
    }

    /// Checks the color of a `CueLed`, updating the self struct instance with
    /// the new value upon success.
    pub fn check_color(&mut self) -> Result<(), CheckColorError> {
        let mut vh = CuePropertyValueHolder::new_with_initial_value(ffi::CorsairLedColor {
            ledId: self.id.into(),
            r: -1,
            g: -1,
            b: -1,
        });
        let was_successful = unsafe {
            ffi::CorsairGetLedsColorsByDeviceIndex(self.device_index as i32, 1, vh.mut_ptr())
        };
        if was_successful {
            let color = LedColor::from_ffi(&vh.value()).map_err(|e| LedColorError(e))?;
            self.last_checked_color = Some(color);
            Ok(())
        } else {
            self.last_checked_color = None;
            Err(SdkError(get_last_error()))
        }
    }

    /// Update the iCUE SDK color buffer with a new color, and modify our `last_buffed_color`
    /// field if that color buffer update operation was successful.
    pub fn update_color_buffer(&mut self, new_color: LedColor) -> CueSdkErrorResult {
        let mut ffi_color = ffi::CorsairLedColor {
            ledId: self.id.into(),
            r: new_color.red as i32,
            g: new_color.green as i32,
            b: new_color.blue as i32,
        };

        let was_successful = unsafe {
            ffi::CorsairSetLedsColorsBufferByDeviceIndex(
                self.device_index as i32,
                1,
                &mut ffi_color as *mut ffi::CorsairLedColor,
            )
        };

        if was_successful {
            self.last_buffed_color = Some(new_color);
            Ok(())
        } else {
            Err(get_last_error())
        }
    }
}

/// A basic RGB struct for describing an LED color.
///
/// It's recommended to reuse these if it makes sense with your use-case, as nothing in the crate
/// ever requires more than a reference (no mutable references,
/// or pass ownership with a value ever required).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LedColor {
    pub red: u8,
    pub blue: u8,
    pub green: u8,
}

/// An error for if the iCUE SDK returns invalid color values that aren't between 0-255.
#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(
    display = "Received invalid color values, should all be between 0-255, instead received red: {}, green: {}, blue:{}",
    _0, _1, _2
)]
pub struct LedColorFromFfiError(i32, i32, i32);

const VALUE_RGB_RANGE: &Range<i32> = &(0..256);

impl LedColor {
    pub(crate) fn from_ffi(color: &ffi::CorsairLedColor) -> Result<Self, LedColorFromFfiError> {
        if !VALUE_RGB_RANGE.contains(&color.r)
            || !VALUE_RGB_RANGE.contains(&color.b)
            || !VALUE_RGB_RANGE.contains(&color.g)
        {
            Err(LedColorFromFfiError(color.r, color.b, color.g))
        } else {
            Ok(LedColor {
                red: color.r as u8,
                blue: color.b as u8,
                green: color.g as u8,
            })
        }
    }
}

/// The position of a given `CueLed`.
///
/// This positions are in physical units for keyboards (inches/cm/mm) and otherwise
/// are "grid" based and not unitized in any "physical" way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LedPosition {
    pub top: f64,
    pub left: f64,
    pub height: f64,
    pub width: f64,
}
