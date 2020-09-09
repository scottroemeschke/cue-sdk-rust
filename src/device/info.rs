use cue_sdk_sys as ffi;
use failure::_core::str::Utf8Error;

use super::{
    channels_from_ffi, Channel, ChannelsFromFfiError, DeviceCapabilities, DeviceId, DeviceLayout,
    DeviceType,
};
use crate::internal::try_c_char_ptr_to_str;

/// The various errors that can occur when reading device info from the iCUE SDK.
#[derive(Debug, Clone, Fail, PartialEq)]
pub enum CueDeviceInfoFromFfiError {
    #[fail(
        display = "Expected to create a CueDevice from a valid pointer, but received a null pointer instead."
    )]
    NullPointer,
    #[fail(display = "Failed to generate on field: {}, error: {}", field, error)]
    StringConversionError {
        field: String,
        #[cause]
        error: Utf8Error,
    },
    #[fail(display = "Unexpected null pointer on field: {}", _0)]
    NullPointerField(String),
    #[fail(display = "Invalid (negative) number of leds: {}", _0)]
    InvalidLedsCount(i32),
    #[fail(display = "Error with channels: {}", _0)]
    ChannelsError(ChannelsFromFfiError),
}

impl CueDeviceInfo {
    pub(crate) fn from_ffi(
        device_info: *mut ffi::CorsairDeviceInfo,
    ) -> Result<Self, CueDeviceInfoFromFfiError> {
        if device_info.is_null() {
            return Err(CueDeviceInfoFromFfiError::NullPointer);
        }

        let info = unsafe { *device_info };
        let id = DeviceId::from_ffi(info.deviceId).map_err(|e| {
            CueDeviceInfoFromFfiError::StringConversionError {
                field: "deviceId".to_owned(),
                error: e.0,
            }
        })?;

        let device_type = DeviceType::from_ffi(info.type_);

        let model = try_c_char_ptr_to_str(info.model)
            .map_err(|e| CueDeviceInfoFromFfiError::StringConversionError {
                field: "model".to_string(),
                error: e,
            })?
            .ok_or_else(|| CueDeviceInfoFromFfiError::NullPointerField("model".to_string()))?
            .to_string();

        let layout = DeviceLayout::from_ffi_values(info.physicalLayout, info.logicalLayout);

        if info.ledsCount < 0 {
            return Err(CueDeviceInfoFromFfiError::InvalidLedsCount(info.ledsCount));
        }

        let leds_count = info.ledsCount as u32;

        let channels = channels_from_ffi(info.channels)
            .map_err(|e| CueDeviceInfoFromFfiError::ChannelsError(e))?;

        let capabilities = DeviceCapabilities::from_ffi(info.capsMask);

        Ok(CueDeviceInfo {
            id,
            capabilities,
            channels,
            leds_count,
            device_type,
            layout,
            model,
        })
    }
}

/// The static device info for the attached `CueDevice`, including id, model, capabilities,
/// leds_count, and more.
#[derive(Debug, Clone, PartialEq)]
pub struct CueDeviceInfo {
    pub id: DeviceId,
    pub device_type: Option<DeviceType>,
    pub model: String,
    pub layout: Option<DeviceLayout>,
    pub capabilities: DeviceCapabilities,
    pub leds_count: u32,
    pub channels: Vec<Channel>,
}

#[cfg(test)]
mod tests {
    use cue_sdk_sys as ffi;

    use std::ffi::CString;
    use std::ptr;

    use super::{CueDeviceInfo, CueDeviceInfoFromFfiError};
    use crate::device::DeviceId;
    use crate::device::{
        Channel, DeviceCapabilities, DeviceLayout, DeviceType, LogicalLayout, PhysicalLayout,
    };
    use std::os::raw::c_char;

    const DEVICE_ID: [c_char; 128] = [
        0x11, 0x11, 0x20, 0x50, 0x30, 0x20, 0x10, 0x50, 0x30, 0x30, 0x30, 0x30, 0x10, 0x11, 0x20,
        0x50, 0x30, 0x20, 0x10, 0x50, 0x30, 0x30, 0x30, 0x30, 0x10, 0x11, 0x20, 0x50, 0x30, 0x20,
        0x10, 0x50, 0x30, 0x30, 0x30, 0x30, 0x10, 0x11, 0x20, 0x50, 0x30, 0x20, 0x10, 0x50, 0x30,
        0x30, 0x30, 0x30, 0x10, 0x11, 0x20, 0x50, 0x30, 0x20, 0x10, 0x50, 0x30, 0x30, 0x30, 0x30,
        0x10, 0x11, 0x20, 0x50, 0x30, 0x20, 0x10, 0x50, 0x30, 0x30, 0x30, 0x30, 0x10, 0x11, 0x20,
        0x50, 0x30, 0x20, 0x10, 0x50, 0x30, 0x30, 0x30, 0x30, 0x10, 0x11, 0x20, 0x50, 0x30, 0x20,
        0x10, 0x50, 0x30, 0x30, 0x30, 0x30, 0x10, 0x11, 0x20, 0x50, 0x30, 0x20, 0x10, 0x50, 0x30,
        0x30, 0x30, 0x30, 0x10, 0x11, 0x20, 0x50, 0x30, 0x20, 0x10, 0x50, 0x30, 0x30, 0x30, 0x30,
        0x10, 0x11, 0x20, 0x50, 0x30, 0x20, 0x10, 0x50,
    ];

    #[test]
    fn device_from_ffi_null_ptr() {
        let result = CueDeviceInfo::from_ffi(ptr::null_mut());
        assert_eq!(result.unwrap_err(), CueDeviceInfoFromFfiError::NullPointer);
    }

    #[test]
    fn device_from_ffi_model_null_ptr() {
        let channels_info = ffi::CorsairChannelsInfo {
            channelsCount: 0,
            channels: ptr::null_mut(),
        };

        let mut info = ffi::CorsairDeviceInfo {
            type_: ffi::CorsairDeviceType_CDT_Cooler,
            model: ptr::null(),
            physicalLayout: ffi::CorsairPhysicalLayout_CPL_BR,
            logicalLayout: ffi::CorsairLogicalLayout_CLL_BR,
            capsMask: 0,
            ledsCount: 20,
            channels: channels_info,
            deviceId: DEVICE_ID,
        };

        let info_ptr: *mut ffi::CorsairDeviceInfo = &mut info;

        let result = CueDeviceInfo::from_ffi(info_ptr);
        assert!(
            matches!(result.unwrap_err(), CueDeviceInfoFromFfiError::NullPointerField(field) if field == "model")
        )
    }

    #[test]
    fn device_from_ffi_model_invalid_utf8() {
        let invalid_utf8 = CString::new([0xC0, 0xC0, 0xC0, 0xC0]).unwrap();

        let channels_info = ffi::CorsairChannelsInfo {
            channelsCount: 0,
            channels: ptr::null_mut(),
        };

        let mut info = ffi::CorsairDeviceInfo {
            type_: ffi::CorsairDeviceType_CDT_Cooler,
            model: invalid_utf8.as_ptr(),
            physicalLayout: ffi::CorsairPhysicalLayout_CPL_BR,
            logicalLayout: ffi::CorsairLogicalLayout_CLL_BR,
            capsMask: 0,
            ledsCount: 20,
            channels: channels_info,
            deviceId: DEVICE_ID,
        };

        let info_ptr: *mut ffi::CorsairDeviceInfo = &mut info;

        let result = CueDeviceInfo::from_ffi(info_ptr);
        assert!(
            matches!(result.unwrap_err(), CueDeviceInfoFromFfiError::StringConversionError {field, ..} if field == "model")
        )
    }

    #[test]
    fn device_from_ffi_invalid_leds_count() {
        let cool_device_model = CString::new("some-cool-device-model").unwrap();

        let channels_info = ffi::CorsairChannelsInfo {
            channelsCount: 0,
            channels: ptr::null_mut(),
        };

        let mut info = ffi::CorsairDeviceInfo {
            type_: ffi::CorsairDeviceType_CDT_Cooler,
            model: cool_device_model.as_ptr(),
            physicalLayout: ffi::CorsairPhysicalLayout_CPL_BR,
            logicalLayout: ffi::CorsairLogicalLayout_CLL_BR,
            capsMask: 0,
            ledsCount: -1,
            channels: channels_info,
            deviceId: DEVICE_ID,
        };

        let info_ptr: *mut ffi::CorsairDeviceInfo = &mut info;

        let result = CueDeviceInfo::from_ffi(info_ptr);
        assert_eq!(
            result.unwrap_err(),
            CueDeviceInfoFromFfiError::InvalidLedsCount(-1)
        );
    }

    #[test]
    fn device_from_ffi_invalid_channels() {
        let cool_device_model = CString::new("some-cool-device-model").unwrap();

        let channels_info = ffi::CorsairChannelsInfo {
            channelsCount: -1,
            channels: ptr::null_mut(),
        };

        let mut info = ffi::CorsairDeviceInfo {
            type_: ffi::CorsairDeviceType_CDT_Cooler,
            model: cool_device_model.as_ptr(),
            physicalLayout: ffi::CorsairPhysicalLayout_CPL_BR,
            logicalLayout: ffi::CorsairLogicalLayout_CLL_BR,
            capsMask: 0,
            ledsCount: 23,
            channels: channels_info,
            deviceId: DEVICE_ID,
        };

        let info_ptr: *mut ffi::CorsairDeviceInfo = &mut info;

        let result = CueDeviceInfo::from_ffi(info_ptr);
        assert!(matches!(
            result.unwrap_err(),
            CueDeviceInfoFromFfiError::ChannelsError(_)
        ));
    }

    #[test]
    fn device_from_ffi_all_valid() {
        let cool_device_model = CString::new("some-cool-device-model").unwrap();

        let mut channel_devices = [
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_QL_Fan,
                deviceLedCount: 6,
            },
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_Strip,
                deviceLedCount: 6,
            },
        ];

        let mut channels = [
            ffi::CorsairChannelInfo {
                totalLedsCount: 2,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
            ffi::CorsairChannelInfo {
                totalLedsCount: 23,
                devicesCount: 0,
                devices: channel_devices.as_mut_ptr(),
            },
        ];

        let channels_info = ffi::CorsairChannelsInfo {
            channelsCount: 2,
            channels: channels.as_mut_ptr(),
        };

        let mut info = ffi::CorsairDeviceInfo {
            type_: ffi::CorsairDeviceType_CDT_Cooler,
            model: cool_device_model.as_ptr(),
            physicalLayout: ffi::CorsairPhysicalLayout_CPL_JP,
            logicalLayout: ffi::CorsairLogicalLayout_CLL_JP,
            capsMask: 0,
            ledsCount: 32,
            channels: channels_info,
            deviceId: DEVICE_ID,
        };

        let info_ptr: *mut ffi::CorsairDeviceInfo = &mut info;

        let result = CueDeviceInfo::from_ffi(info_ptr);
        assert_eq!(result.unwrap(), CueDeviceInfo {
            id: DeviceId("\u{11}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P0000\u{10}\u{11} P0 \u{10}P".to_string()),
            device_type: Some(DeviceType::Cooler),
            model: "some-cool-device-model".to_string(),
            layout: Some(DeviceLayout::Keyboard {
                physical_layout: PhysicalLayout::KeyboardJp,
                logical_layout: LogicalLayout::KeyboardJp
            }),
            capabilities: DeviceCapabilities {
                lighting: false,
                property_lookup: false
            },
            leds_count: 32,
            channels: vec![
                Channel {
                    total_led_count: 2,
                    devices: vec![]
                },
                Channel {
                    total_led_count: 23,
                    devices: vec![]
                }
            ]
        });
    }
}
