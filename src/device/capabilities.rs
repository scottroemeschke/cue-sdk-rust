use cue_sdk_sys as ffi;

/// The known capabilities of the `CueDevice`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCapabilities {
    pub lighting: bool,
    pub property_lookup: bool,
}

impl DeviceCapabilities {
    pub(crate) fn from_ffi(bitmask: i32) -> Self {
        let lighting_val = bitmask & ffi::CorsairDeviceCaps_CDC_Lighting as i32;
        let property_lookup_val = bitmask & ffi::CorsairDeviceCaps_CDC_PropertyLookup as i32;
        DeviceCapabilities {
            lighting: lighting_val != 0,
            property_lookup: property_lookup_val != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use cue_sdk_sys as ffi;

    use super::DeviceCapabilities;

    #[test]
    pub fn from_ffi_both_enabled() {
        let bitmask = 0i32
            | ffi::CorsairDeviceCaps_CDC_Lighting as i32
            | ffi::CorsairDeviceCaps_CDC_PropertyLookup as i32;
        assert_eq!(
            DeviceCapabilities::from_ffi(bitmask),
            DeviceCapabilities {
                property_lookup: true,
                lighting: true
            }
        )
    }

    #[test]
    pub fn from_ffi_neither_enabled() {
        let bitmask = 0i32;
        assert_eq!(
            DeviceCapabilities::from_ffi(bitmask),
            DeviceCapabilities {
                property_lookup: false,
                lighting: false
            }
        )
    }

    #[test]
    pub fn from_ffi_just_lighting() {
        let bitmask = 0i32 | ffi::CorsairDeviceCaps_CDC_Lighting as i32;
        assert_eq!(
            DeviceCapabilities::from_ffi(bitmask),
            DeviceCapabilities {
                property_lookup: false,
                lighting: true
            }
        )
    }

    #[test]
    pub fn from_ffi_just_property_lookup() {
        let bitmask = 0i32 | ffi::CorsairDeviceCaps_CDC_PropertyLookup as i32;
        assert_eq!(
            DeviceCapabilities::from_ffi(bitmask),
            DeviceCapabilities {
                property_lookup: true,
                lighting: false
            }
        )
    }
}
