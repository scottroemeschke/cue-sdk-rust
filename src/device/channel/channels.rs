use cue_sdk_sys as ffi;

use std::ptr::slice_from_raw_parts;

use super::{Channel, ChannelFromFfiError};

/// The various errors that can happen when reading all of the channels in a given `DeviceInfo` from the
/// iCUE SDK.
#[derive(Debug, Clone, Fail, PartialEq)]
pub enum ChannelsFromFfiError {
    #[fail(
        display = "Received invalid (negative) count of channels, count: {}",
        _0
    )]
    InvalidChannelsCount(i32),
    #[fail(display = "Channels pointer was null.")]
    NullChannelsPointer,
    #[fail(display = "Channels from ffi failed: {:?}", _0)]
    ChannelFromFFIErrors(Vec<ChannelFromFfiError>),
}

pub(crate) fn channels_from_ffi(
    info: ffi::CorsairChannelsInfo,
) -> Result<Vec<Channel>, ChannelsFromFfiError> {
    if info.channelsCount == 0 {
        return Ok(vec![]);
    }
    if info.channelsCount < 0 {
        return Err(ChannelsFromFfiError::InvalidChannelsCount(
            info.channelsCount,
        ));
    }
    if info.channels.is_null() {
        return Err(ChannelsFromFfiError::NullChannelsPointer);
    }

    let num_channels = info.channelsCount as usize;

    let channels = unsafe { &(*slice_from_raw_parts(info.channels, num_channels)) };

    let mut errs = Vec::<ChannelFromFfiError>::with_capacity(num_channels);
    let mut channels_out = Vec::<Channel>::with_capacity(num_channels);

    for c in channels.iter() {
        match Channel::from_ffi(c) {
            Ok(channel) => channels_out.push(channel),
            Err(e) => errs.push(e),
        }
    }

    if errs.is_empty() {
        Ok(channels_out)
    } else {
        Err(ChannelsFromFfiError::ChannelFromFFIErrors(errs))
    }
}

#[cfg(test)]
mod tests {
    use cue_sdk_sys as ffi;

    use std::ptr;

    use super::Channel;

    use super::super::{channels_from_ffi, ChannelDevice, ChannelDeviceType, ChannelsFromFfiError};

    #[test]
    fn channels_from_ffi_channels_count_zero() {
        let info = ffi::CorsairChannelsInfo {
            channelsCount: 0,
            channels: ptr::null_mut(),
        };

        let result = channels_from_ffi(info);
        assert_eq!(result.unwrap(), vec![]);
    }

    #[test]
    fn channels_from_ffi_invalid_channels_count() {
        let info = ffi::CorsairChannelsInfo {
            channelsCount: -1,
            channels: ptr::null_mut(),
        };

        let result = channels_from_ffi(info);
        assert_eq!(
            result.unwrap_err(),
            ChannelsFromFfiError::InvalidChannelsCount(-1)
        );
    }

    #[test]
    fn channels_from_ffi_channels_ptr_null() {
        let info = ffi::CorsairChannelsInfo {
            channelsCount: 5,
            channels: ptr::null_mut(),
        };

        let result = channels_from_ffi(info);
        assert_eq!(
            result.unwrap_err(),
            ChannelsFromFfiError::NullChannelsPointer
        );
    }

    #[test]
    fn channels_from_ffi_channels_some_channels_invalid() {
        let mut chans = [
            ffi::CorsairChannelInfo {
                totalLedsCount: 23,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
            ffi::CorsairChannelInfo {
                totalLedsCount: -1,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
        ];

        let info = ffi::CorsairChannelsInfo {
            channelsCount: 2,
            channels: chans.as_mut_ptr(),
        };

        let result = channels_from_ffi(info);
        assert!(
            matches!(result.unwrap_err(), ChannelsFromFfiError::ChannelFromFFIErrors(errs) if errs.len() == 1)
        );
    }

    #[test]
    fn channels_from_ffi_channels_all_channels_invalid() {
        let mut chans = [
            ffi::CorsairChannelInfo {
                totalLedsCount: -234,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
            ffi::CorsairChannelInfo {
                totalLedsCount: -1,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
            ffi::CorsairChannelInfo {
                totalLedsCount: -5,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
            ffi::CorsairChannelInfo {
                totalLedsCount: -25,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
        ];

        let info = ffi::CorsairChannelsInfo {
            channelsCount: 4,
            channels: chans.as_mut_ptr(),
        };

        let result = channels_from_ffi(info);
        assert!(
            matches!(result.unwrap_err(), ChannelsFromFfiError::ChannelFromFFIErrors(errs) if errs.len() == 4)
        );
    }

    #[test]
    fn channels_from_ffi_channels_all_valid() {
        let mut devices = [
            ffi::CorsairChannelDeviceInfo {
                deviceLedCount: 43,
                type_: ffi::CorsairChannelDeviceType_CCDT_ML_Fan,
            },
            ffi::CorsairChannelDeviceInfo {
                deviceLedCount: 2,
                type_: ffi::CorsairChannelDeviceType_CCDT_QL_Fan,
            },
        ];

        let mut chans = [
            ffi::CorsairChannelInfo {
                totalLedsCount: 23,
                devicesCount: 0,
                devices: ptr::null_mut(),
            },
            ffi::CorsairChannelInfo {
                totalLedsCount: 9,
                devicesCount: 2,
                devices: devices.as_mut_ptr(),
            },
        ];

        let info = ffi::CorsairChannelsInfo {
            channelsCount: 2,
            channels: chans.as_mut_ptr(),
        };

        let result = channels_from_ffi(info);
        assert_eq!(
            result.unwrap(),
            vec![
                Channel {
                    total_led_count: 23,
                    devices: vec![],
                },
                Channel {
                    total_led_count: 9,
                    devices: vec![
                        ChannelDevice {
                            device_type: Some(ChannelDeviceType::MlFan),
                            device_led_count: 43,
                        },
                        ChannelDevice {
                            device_type: Some(ChannelDeviceType::QlFan),
                            device_led_count: 2,
                        }
                    ],
                }
            ]
        );
    }
}
