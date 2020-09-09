//! Contains the `CueDevice` struct, it's methods, errors, and other "device related" structs and
//! functionality, including channels, device layout and capabilities, and more.
use cue_sdk_sys as ffi;

use std::ptr::slice_from_raw_parts;

use crate::led::{CueLed, LedColor, LedColorFromFfiError};

use crate::errors::{get_last_error, CueSdkError};

mod capabilities;
mod channel;
mod device_type;
mod id;
mod info;
mod layout;

use crate::internal::CuePropertyValueHolder;
use crate::property::{BooleanProperty, BooleanPropertyKey, Int32Property, Int32PropertyKey};

pub use capabilities::DeviceCapabilities;
pub use channel::{
    Channel, ChannelDevice, ChannelDeviceFromFfiError, ChannelDeviceType, ChannelFromFfiError,
    ChannelsFromFfiError,
};
pub use device_type::DeviceType;
pub use id::{DeviceId, DeviceIdFromFfiError};
pub use info::{CueDeviceInfo, CueDeviceInfoFromFfiError};
pub use layout::{DeviceLayout, LogicalLayout, PhysicalLayout};

pub(crate) use channel::channels_from_ffi;

pub type DeviceIndex = u32;
pub type DeviceCount = u32;

/// The `CueDevice` holds the static device information, the `device_index` which much of the
/// iCUE SDK depends on. And the `CueLed`s on the device which can be interacted with directly
/// or interacted with "via" the `CueDevice` itself.
#[derive(Debug, Clone, PartialEq)]
pub struct CueDevice {
    pub device_info: CueDeviceInfo,
    pub device_index: DeviceIndex,
    pub leds: Vec<CueLed>,
}

/// The error that can be returned when we try to create a `CueDevice` from existing `CueDeviceInfo`.
#[derive(Debug, Clone, Fail, PartialEq)]
#[fail(
    display = "Failed to crate a cue device from device info and index, failed to get led positions: {:?}.",
    _0
)]
pub struct CueDeviceFromDeviceInfoAndIndexError(GetLedPositionsError);

impl CueDevice {
    pub(crate) fn from_device_info_and_index(
        device_index: DeviceIndex,
        device_info: CueDeviceInfo,
    ) -> Result<Self, CueDeviceFromDeviceInfoAndIndexError> {
        let leds = get_leds_for_device_index(device_index)
            .map_err(|e| CueDeviceFromDeviceInfoAndIndexError(e))?;
        Ok(CueDevice {
            device_info,
            leds,
            device_index,
        })
    }
}

fn get_leds_for_device_index(index: DeviceIndex) -> Result<Vec<CueLed>, GetLedPositionsError> {
    let ffi_leds_ptr = unsafe { ffi::CorsairGetLedPositionsByDeviceIndex(index as i32) };
    if ffi_leds_ptr.is_null() {
        return Err(GetLedPositionsError(get_last_error()));
    }

    let ffi_leds = unsafe {
        let ffi_led_positions = *ffi_leds_ptr;
        &*(slice_from_raw_parts(
            ffi_led_positions.pLedPosition,
            ffi_led_positions.numberOfLed as usize,
        ))
    };

    Ok(ffi_leds
        .iter()
        .map(|led| CueLed::with_device_data_from_ffi(index, led))
        .collect::<Vec<CueLed>>())
}

fn get_led_colors(leds: &mut Vec<CueLed>, index: DeviceIndex) -> Result<(), GetLedColorsError> {
    let mut led_colors = leds
        .iter()
        .map(|c| ffi::CorsairLedColor {
            ledId: c.id.into(),
            r: -1,
            g: -1,
            b: -1,
        })
        .collect::<Vec<ffi::CorsairLedColor>>();

    let was_successful = unsafe {
        ffi::CorsairGetLedsColorsByDeviceIndex(
            index as i32,
            leds.len() as i32,
            led_colors.as_mut_ptr(),
        )
    };

    if !was_successful {
        return Err(GetLedColorsError::SdkFailure(get_last_error()));
    }

    let mut errs = Vec::<LedColorFromFfiError>::with_capacity(leds.len());

    leds.iter_mut()
        .zip(led_colors.into_iter())
        .for_each(|(a, b)| match LedColor::from_ffi(&b) {
            Ok(c) => a.last_checked_color = Some(c),
            Err(e) => errs.push(e),
        });

    if errs.is_empty() {
        Ok(())
    } else {
        Err(GetLedColorsError::InvalidLedColorValues(errs))
    }
}

/// The error that can be returned when we fail to get a `BooleanProperty` on a `CueDevice`.
#[derive(Debug, Clone, PartialEq, Fail)]
pub enum GetPropertyError {
    #[fail(
        display = "Cannot get {} property: {}, since the device does not support property lookup functionality.",
        _0, _1
    )]
    NoPropertyLookupSupport {
        property_type: String,
        property_key_name: String,
        property_key_value: u32,
    },
    #[fail(display = "SDK failed, error: {:?}", _0)]
    SdkErr(Option<CueSdkError>),
}

/// The error that can occur when getting led positions fails for a given `CueDevice`.
#[derive(Debug, Clone, PartialEq, Fail)]
#[fail(
    display = "Received null pointer from native CUESDK when getting led positions, with error: {:?}",
    _0
)]
pub struct GetLedPositionsError(Option<CueSdkError>);

/// The error that can occur when getting led information fails for a given `CueDevice`.
#[derive(Debug, Clone, PartialEq, Fail)]
pub enum GetLedsError {
    #[fail(display = "Failed to get led positions, with error: {:?}", _0)]
    GetPositionsError(GetLedPositionsError),
    #[fail(display = "Failed to get led colors, with error: {:?}", _0)]
    GetColorsError(GetLedColorsError),
}

// The error that can occur when getting led color information fails for a given `CueDevice`.
#[derive(Debug, Clone, PartialEq, Fail)]
pub enum GetLedColorsError {
    #[fail(display = "Failed to get led colors, with error: {:?}", _0)]
    SdkFailure(Option<CueSdkError>),
    #[fail(display = "Received invalid color values, errors: {:?}", _0)]
    InvalidLedColorValues(Vec<LedColorFromFfiError>),
}

impl CueDevice {
    /// Refreshes the led colors for the specified device, returning an error
    /// if that operation fails.
    pub fn refresh_leds_colors(&mut self) -> Result<(), GetLedsError> {
        get_led_colors(&mut self.leds, self.device_index)
            .map_err(|e| GetLedsError::GetColorsError(e))?;
        Ok(())
    }

    /// Attempts to get the `BooleanProperty` with the specified `BooleanPropertyKey`.
    pub fn get_bool_property(
        &self,
        key: BooleanPropertyKey,
    ) -> Result<BooleanProperty, GetPropertyError> {
        if !self.device_info.capabilities.property_lookup {
            return Err(GetPropertyError::NoPropertyLookupSupport {
                property_type: "Bool".to_string(),
                property_key_name: format!("{:?}", key),
                property_key_value: key.into(),
            });
        }

        let mut vh: CuePropertyValueHolder<bool> = CuePropertyValueHolder::new();
        let was_successful = unsafe {
            ffi::CorsairGetBoolPropertyValue(self.device_index as i32, key as u32, vh.mut_ptr())
        };

        if was_successful {
            Ok(BooleanProperty::new(self.device_index, key, vh.value()))
        } else {
            Err(GetPropertyError::SdkErr(get_last_error()))
        }
    }

    /// Attempts to get the `Int32Property` with the specified `Int32PropertyKey`.
    pub fn get_int32_property(
        &self,
        key: Int32PropertyKey,
    ) -> Result<Int32Property, GetPropertyError> {
        if !self.device_info.capabilities.property_lookup {
            return Err(GetPropertyError::NoPropertyLookupSupport {
                property_type: "Int32".to_string(),
                property_key_name: format!("{:?}", key),
                property_key_value: key.into(),
            });
        }

        let mut vh = CuePropertyValueHolder::<i32>::new();
        let was_successful = unsafe {
            ffi::CorsairGetInt32PropertyValue(self.device_index as i32, key as u32, vh.mut_ptr())
        };

        if was_successful {
            Ok(Int32Property::new(self.device_index, key, vh.value()))
        } else {
            Err(GetPropertyError::SdkErr(get_last_error()))
        }
    }
}
