use cue_sdk_sys as ffi;
use num_traits::FromPrimitive;

/// The various device types supported by the iCUE SDK.
#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u32)]
pub enum DeviceType {
    Mouse = ffi::CorsairDeviceType_CDT_Mouse,
    Keyboard = ffi::CorsairDeviceType_CDT_Keyboard,
    Headset = ffi::CorsairDeviceType_CDT_Headset,
    MouseMat = ffi::CorsairDeviceType_CDT_MouseMat,
    HeadsetStand = ffi::CorsairDeviceType_CDT_HeadsetStand,
    CommanderPro = ffi::CorsairDeviceType_CDT_CommanderPro,
    LightingNodePro = ffi::CorsairDeviceType_CDT_LightingNodePro,
    MemoryModule = ffi::CorsairDeviceType_CDT_MemoryModule,
    Cooler = ffi::CorsairDeviceType_CDT_Cooler,
    Motherboard = ffi::CorsairDeviceType_CDT_Motherboard,
    GraphicsCard = ffi::CorsairDeviceType_CDT_GraphicsCard,
}

impl DeviceType {
    pub(crate) fn from_ffi(device_type: ffi::CorsairDeviceType) -> Option<Self> {
        DeviceType::from_u32(device_type)
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceType;
    use cue_sdk_sys as ffi;

    #[test]
    fn from_ffi_valid() {
        assert_eq!(
            DeviceType::from_ffi(ffi::CorsairDeviceType_CDT_Cooler),
            Some(DeviceType::Cooler)
        );
    }

    #[test]
    fn from_ffi_invalid() {
        assert_eq!(
            DeviceType::from_ffi(ffi::CorsairDeviceType_CDT_Unknown),
            None
        );
        assert_eq!(DeviceType::from_ffi(1642), None);
    }
}
