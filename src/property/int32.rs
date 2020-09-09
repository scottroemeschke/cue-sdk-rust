use cue_sdk_sys as ffi;
use num_traits::ToPrimitive;

use super::RefreshValueError;
use crate::device::DeviceIndex;
use crate::errors::get_last_error;
use crate::internal::CuePropertyValueHolder;

/// An `int32` property that can be refreshed to "check" the property at any point.
#[derive(Debug, Clone, PartialEq)]
pub struct Int32Property {
    pub key: Int32PropertyKey,
    device_index: DeviceIndex,
    pub last_value: i32,
}

impl Int32Property {
    pub(crate) fn new(
        device_index: DeviceIndex,
        key: Int32PropertyKey,
        initial_value: i32,
    ) -> Self {
        Int32Property {
            device_index,
            key,
            last_value: initial_value,
        }
    }

    pub fn refresh_value(&mut self) -> Result<(), RefreshValueError> {
        let mut new_value_holder = CuePropertyValueHolder::<i32>::new();
        let was_successful = unsafe {
            ffi::CorsairGetInt32PropertyValue(
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

/// The valid keys that some devices support for `int32` property lookups.
#[derive(Debug, Clone, Copy, ToPrimitive, FromPrimitive, PartialEq)]
#[cfg_attr(test, derive(EnumIter))]
pub enum Int32PropertyKey {
    HeadsetEqualizerPreset = ffi::CorsairDevicePropertyId_CDPI_Headset_EqualizerPreset as isize,
}

impl From<Int32PropertyKey> for u32 {
    fn from(key: Int32PropertyKey) -> Self {
        //this unwrap is covered for all variants in unit tests
        key.to_u32().unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use strum::IntoEnumIterator;

    use super::{Int32Property, Int32PropertyKey};

    #[test]
    fn new() {
        let prop = Int32Property::new(2, Int32PropertyKey::HeadsetEqualizerPreset, 12);
        assert_eq!(
            prop,
            Int32Property {
                device_index: 2,
                key: Int32PropertyKey::HeadsetEqualizerPreset,
                last_value: 12
            }
        )
    }

    #[test]
    fn from_boolean_property_key_for_u32() {
        for key in Int32PropertyKey::iter() {
            u32::from(key); //ensure this never panics
        }
    }
}
