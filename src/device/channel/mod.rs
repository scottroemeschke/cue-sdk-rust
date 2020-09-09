mod channels;
mod device;

use cue_sdk_sys as ffi;

use std::convert::TryInto;
use std::ptr::slice_from_raw_parts;

pub(crate) use channels::channels_from_ffi;
pub use channels::ChannelsFromFfiError;
pub use device::{ChannelDevice, ChannelDeviceFromFfiError, ChannelDeviceType};

/// The iCUE platform has two main types of products. Those that are "channel-less" like
/// mice, keyboards, etc. And those that have "channels" which are things like CPU Coolers, and what
/// they call "DIY" devices (think LED strips you can buy and customize or combine).
///
/// This struct represents a "channel" on one of the second type of devices listed above, which
/// has it's own "devices" list.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub total_led_count: u32,
    pub devices: Vec<ChannelDevice>,
}

/// The various errors that can happen when reading a channel from the iCUE SDK.
#[derive(Debug, Clone, Fail, PartialEq)]
pub enum ChannelFromFfiError {
    #[fail(display = "Invalid led count: {}", _0)]
    InvalidLedCount(i32),
    #[fail(display = "Invalid num_devices array size: {}", _0)]
    InvalidNumDevices(i32),
    #[fail(display = "devices array pointer is null")]
    DevicesNullPtr,
    #[fail(display = "ChannelDeviceErrors: {:?}", _0)]
    ChannelDeviceErrors(Vec<ChannelDeviceFromFfiError>),
}

impl Channel {
    pub(crate) fn from_ffi(
        channel_info: &ffi::CorsairChannelInfo,
    ) -> Result<Self, ChannelFromFfiError> {
        if channel_info.totalLedsCount < 0 {
            return Err(ChannelFromFfiError::InvalidLedCount(
                channel_info.totalLedsCount,
            ));
        }

        let total_led_count = channel_info.totalLedsCount.try_into().unwrap();

        let num_devices = channel_info.devicesCount;

        if num_devices == 0 {
            return Ok(Channel {
                total_led_count,
                devices: vec![],
            });
        }
        if num_devices < 0 {
            return Err(ChannelFromFfiError::InvalidNumDevices(num_devices));
        }
        if channel_info.devices.is_null() {
            return Err(ChannelFromFfiError::DevicesNullPtr);
        }

        let num_devices_positive: u32 = num_devices.try_into().unwrap();

        let channel_device_infos = unsafe {
            &(*slice_from_raw_parts(channel_info.devices, num_devices_positive as usize))
        };

        let mut errs =
            Vec::<ChannelDeviceFromFfiError>::with_capacity(num_devices_positive as usize);
        let mut devices = Vec::<ChannelDevice>::with_capacity(num_devices_positive as usize);
        for d in channel_device_infos {
            match ChannelDevice::from_ffi(d) {
                Ok(cd) => devices.push(cd),
                Err(e) => errs.push(e),
            }
        }

        if !errs.is_empty() {
            Err(ChannelFromFfiError::ChannelDeviceErrors(errs))
        } else {
            Ok(Channel {
                total_led_count,
                devices,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use cue_sdk_sys as ffi;

    use std::ptr;

    use super::{Channel, ChannelDevice, ChannelDeviceType, ChannelFromFfiError};

    #[test]
    fn channel_from_ffi_total_leds_count_invalid() {
        let info = ffi::CorsairChannelInfo {
            totalLedsCount: -1,
            devicesCount: 20,
            devices: ptr::null_mut(),
        };
        let result = Channel::from_ffi(&info);
        assert_eq!(
            result.unwrap_err(),
            ChannelFromFfiError::InvalidLedCount(-1)
        );
    }

    #[test]
    fn channel_from_ffi_no_channel_devices() {
        let info = ffi::CorsairChannelInfo {
            totalLedsCount: 34,
            devicesCount: 0,
            devices: ptr::null_mut(),
        };
        let result = Channel::from_ffi(&info);
        assert_eq!(
            result.unwrap(),
            Channel {
                devices: vec![],
                total_led_count: 34,
            }
        );
    }

    #[test]
    fn channel_from_ffi_device_count_invalid() {
        let info = ffi::CorsairChannelInfo {
            totalLedsCount: 98,
            devicesCount: -1,
            devices: ptr::null_mut(),
        };
        let result = Channel::from_ffi(&info);
        assert_eq!(
            result.unwrap_err(),
            ChannelFromFfiError::InvalidNumDevices(-1)
        );
    }

    #[test]
    fn channel_from_ffi_devices_ptr_null() {
        let info = ffi::CorsairChannelInfo {
            totalLedsCount: 12,
            devicesCount: 14,
            devices: ptr::null_mut(),
        };
        let result = Channel::from_ffi(&info);
        assert_eq!(result.unwrap_err(), ChannelFromFfiError::DevicesNullPtr);
    }

    #[test]
    fn channel_from_ffi_some_devices_invalid() {
        let mut devices = [
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_HD_Fan,
                deviceLedCount: 982,
            },
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_HD_Fan,
                deviceLedCount: -50,
            },
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_Pump,
                deviceLedCount: -100,
            },
        ];

        let info = ffi::CorsairChannelInfo {
            totalLedsCount: 12,
            devicesCount: 3,
            devices: devices.as_mut_ptr(),
        };

        let result = Channel::from_ffi(&info);
        assert!(
            matches!(result.unwrap_err(), ChannelFromFfiError::ChannelDeviceErrors(errs) if errs.len() == 2)
        );
    }

    #[test]
    fn channel_from_ffi_all_devices_invalid() {
        let mut devices = [
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_HD_Fan,
                deviceLedCount: -1,
            },
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_HD_Fan,
                deviceLedCount: -50,
            },
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_Pump,
                deviceLedCount: -100,
            },
        ];

        let info = ffi::CorsairChannelInfo {
            totalLedsCount: 12,
            devicesCount: 3,
            devices: devices.as_mut_ptr(),
        };

        let result = Channel::from_ffi(&info);
        assert!(
            matches!(result.unwrap_err(), ChannelFromFfiError::ChannelDeviceErrors(errs) if errs.len() == 3)
        );
    }

    #[test]
    fn channel_from_ffi_all_valid() {
        let mut devices = [
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_HD_Fan,
                deviceLedCount: 40,
            },
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_HD_Fan,
                deviceLedCount: 80,
            },
            ffi::CorsairChannelDeviceInfo {
                type_: ffi::CorsairChannelDeviceType_CCDT_Pump,
                deviceLedCount: 3,
            },
        ];

        let info = ffi::CorsairChannelInfo {
            totalLedsCount: 12,
            devicesCount: 3,
            devices: devices.as_mut_ptr(),
        };

        let result = Channel::from_ffi(&info);
        assert_eq!(
            result.unwrap(),
            Channel {
                total_led_count: 12,
                devices: vec![
                    ChannelDevice {
                        device_led_count: 40,
                        device_type: Some(ChannelDeviceType::HdFan)
                    },
                    ChannelDevice {
                        device_led_count: 80,
                        device_type: Some(ChannelDeviceType::HdFan)
                    },
                    ChannelDevice {
                        device_led_count: 3,
                        device_type: Some(ChannelDeviceType::Pump)
                    },
                ]
            }
        );
    }
}
