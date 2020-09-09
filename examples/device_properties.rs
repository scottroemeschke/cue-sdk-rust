extern crate cue_sdk;
extern crate env_logger;

use cue_sdk::sdk::GetAllDevicesError;
use env_logger::Env;
use log::info;

use cue_sdk::device::{CueDevice, DeviceType};
use cue_sdk::property::{BooleanPropertyKey, Int32PropertyKey};

pub fn main() {
    env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));
    let sdk = cue_sdk::initialize().expect("failed to initialize sdk");

    let all_devices = sdk
        .get_all_devices()
        .map_err(|e| {
            match &e {
                GetAllDevicesError::GetDeviceCountError(_) => {
                    //the device count call failed for some reason (which we need to know how many devices to ask for)
                }
                GetAllDevicesError::GetDeviceAtIndexErrors(_) => {
                    //some or all devices had failures when getting details about them
                    //the tuple value is a vec of errors which will give you more in-depth info
                }
            }
            e
        })
        .expect("failed to get all devices");

    if all_devices.is_empty() {
        info!("No devices connected...");
    } else {
        info!("Searching for devices with property lookup capabilities...");
        let lookup_devices = all_devices
            .into_iter()
            .filter(|d| d.device_info.capabilities.property_lookup)
            .collect::<Vec<CueDevice>>();

        if lookup_devices.is_empty() {
            info!("No devices found with property lookup functionality.");
        } else {
            for pld in &lookup_devices {
                //Unfortunately, the current CUESDK implementation only tells you whether or not
                // a device supports property lookup, but not which keys it has.

                //You can likely intuit this based on the key names and device types though...
                if let Some(device_type) = pld.device_info.device_type {
                    if let DeviceType::Headset = device_type {
                        info!("Found a headset!");

                        //There are two property types, bools and int32s
                        let mut eq_prop = pld
                            .get_int32_property(Int32PropertyKey::HeadsetEqualizerPreset)
                            .expect("failed to get headset equalizer preset");

                        //these properties have a "last value" property, and the ability to "refresh" the value
                        info!(
                            "The last value for the headset eq preset is: {:?}",
                            eq_prop.last_value
                        );
                        eq_prop
                            .refresh_value()
                            .expect("failed to refresh eq property value");

                        info!(
                            "The new updated value for the headset eq preset is: {:?}",
                            eq_prop.last_value
                        );

                        let mic_enabled = pld
                            .get_bool_property(BooleanPropertyKey::HeadsetMicEnabled)
                            .expect("failed to get headset mic enabled property");

                        info!("Headset mic enable status: {:?}", mic_enabled.last_value);

                        //there is currently no "write" property functionality exposed in the CUESDK
                    }
                }
            }
        }
    }
}
