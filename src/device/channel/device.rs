use cue_sdk_sys as ffi;
use num_traits::FromPrimitive;

/// The types of devices that can be attached to a `Channel`.
#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u32)]
pub enum ChannelDeviceType {
    HdFan = ffi::CorsairChannelDeviceType_CCDT_HD_Fan,
    SpFan = ffi::CorsairChannelDeviceType_CCDT_SP_Fan,
    LlFan = ffi::CorsairChannelDeviceType_CCDT_LL_Fan,
    MlFan = ffi::CorsairChannelDeviceType_CCDT_ML_Fan,
    Strip = ffi::CorsairChannelDeviceType_CCDT_Strip,
    Dap = ffi::CorsairChannelDeviceType_CCDT_DAP,
    Pump = ffi::CorsairChannelDeviceType_CCDT_Pump,
    QlFan = ffi::CorsairChannelDeviceType_CCDT_QL_Fan,
    WaterBlock = ffi::CorsairChannelDeviceType_CCDT_WaterBlock,
    SpProFan = ffi::CorsairChannelDeviceType_CCDT_SPPRO_Fan,
}

/// Not to be confused with a `CueDevice` a `ChannelDevice` is attached to a `Channel` which is a
/// "DIY" style LED channel attached to a proper `CueDevice`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDevice {
    pub device_type: Option<ChannelDeviceType>,
    pub device_led_count: u32,
}

/// The various errors that can happen when reading a `ChannelDevice` from the iCUE SDK.
#[derive(Debug, Clone, Fail, PartialEq)]
#[fail(display = "Invalid device_led_count: {}", device_led_count)]
pub struct ChannelDeviceFromFfiError {
    pub device_led_count: i32,
}

impl ChannelDevice {
    pub(crate) fn from_ffi(
        device_info: &ffi::CorsairChannelDeviceInfo,
    ) -> Result<Self, ChannelDeviceFromFfiError> {
        let led_count = device_info.deviceLedCount;

        if led_count < 0 {
            Err(ChannelDeviceFromFfiError {
                device_led_count: led_count,
            })
        } else {
            let device_led_count: u32 = led_count as u32;
            let device_type = ChannelDeviceType::from_u32(device_info.type_);

            Ok(ChannelDevice {
                device_led_count,
                device_type,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use cue_sdk_sys as ffi;

    use super::{ChannelDevice, ChannelDeviceFromFfiError, ChannelDeviceType};

    #[test]
    fn channel_device_from_ffi_invalid_led_count() {
        let channel_device_info = ffi::CorsairChannelDeviceInfo {
            type_: ffi::CorsairChannelDeviceType_CCDT_Pump,
            deviceLedCount: -1,
        };
        let result = ChannelDevice::from_ffi(&channel_device_info);
        assert_eq!(
            result.unwrap_err(),
            ChannelDeviceFromFfiError {
                device_led_count: -1
            }
        );
    }

    #[test]
    fn channel_device_from_ffi_valid_with_device_type() {
        let channel_device_info = ffi::CorsairChannelDeviceInfo {
            type_: ffi::CorsairChannelDeviceType_CCDT_Strip,
            deviceLedCount: 12,
        };
        let result = ChannelDevice::from_ffi(&channel_device_info);
        assert_eq!(
            result.unwrap(),
            ChannelDevice {
                device_led_count: 12,
                device_type: Some(ChannelDeviceType::Strip),
            }
        );
    }

    #[test]
    fn channel_device_from_ffi_valid_with_unknown_device_type() {
        let channel_device_info = ffi::CorsairChannelDeviceInfo {
            type_: 509,
            deviceLedCount: 13,
        };
        let result = ChannelDevice::from_ffi(&channel_device_info);
        assert_eq!(
            result.unwrap(),
            ChannelDevice {
                device_led_count: 13,
                device_type: None,
            }
        );
    }
}
