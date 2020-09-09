use cue_sdk_sys as ffi;
use num_traits::ToPrimitive;

use super::RefreshValueError;
use crate::device::DeviceIndex;
use crate::errors::get_last_error;
use crate::internal::CuePropertyValueHolder;

/// A `boolean` property that can be refreshed to "check" the property at any point.
#[derive(Debug, Clone, PartialEq)]
pub struct BooleanProperty {
    pub key: BooleanPropertyKey,
    device_index: DeviceIndex,
    pub last_value: bool,
}

impl BooleanProperty {
    pub(crate) fn new(
        device_index: DeviceIndex,
        key: BooleanPropertyKey,
        initial_value: bool,
    ) -> Self {
        BooleanProperty {
            device_index,
            key,
            last_value: initial_value,
        }
    }

    pub fn refresh_value(&mut self) -> Result<(), RefreshValueError> {
        let mut new_value_holder = CuePropertyValueHolder::<bool>::new();
        let was_successful = unsafe {
            ffi::CorsairGetBoolPropertyValue(
                self.device_index as i32,
                self.key.into(),
                new_value_holder.mut_ptr(),
            )
        };
        if was_successful {
            let updated_value = new_value_holder.value();
            self.last_value = updated_value;
            Ok(())
        } else {
            Err(RefreshValueError(get_last_error()))
        }
    }
}

/// The valid keys that some devices support for `boolean` property lookups.
#[derive(Debug, Clone, Copy, ToPrimitive, FromPrimitive, PartialEq)]
#[cfg_attr(test, derive(EnumIter))]
pub enum BooleanPropertyKey {
    HeadsetMicEnabled = ffi::CorsairDevicePropertyId_CDPI_Headset_MicEnabled as isize,
    HeadsetSurroundSoundEnabled =
        ffi::CorsairDevicePropertyId_CDPI_Headset_SurroundSoundEnabled as isize,
    HeadsetSidetoneEnabled = ffi::CorsairDevicePropertyId_CDPI_Headset_SidetoneEnabled as isize,
}

impl From<BooleanPropertyKey> for u32 {
    fn from(key: BooleanPropertyKey) -> Self {
        //this unwrap is covered for all variants in unit tests
        key.to_u32().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::property::{BooleanProperty, BooleanPropertyKey};
    #[cfg(test)]
    use strum::IntoEnumIterator;

    #[test]
    fn new() {
        let prop = BooleanProperty::new(5, BooleanPropertyKey::HeadsetMicEnabled, false);
        assert_eq!(
            prop,
            BooleanProperty {
                device_index: 5,
                key: BooleanPropertyKey::HeadsetMicEnabled,
                last_value: false
            }
        )
    }

    #[test]
    fn from_boolean_property_key_for_u32() {
        for key in BooleanPropertyKey::iter() {
            u32::from(key); //ensure this never panics
        }
    }
}
