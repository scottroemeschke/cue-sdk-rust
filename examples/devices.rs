extern crate cue_sdk;
extern crate env_logger;

use cue_sdk::sdk::GetAllDevicesError;
use env_logger::Env;
use log::info;

pub fn main() {
    env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));
    let sdk = cue_sdk::initialize().expect("failed to initialize sdk");

    info!("Requesting device count...");
    let count = sdk.get_device_count().expect("failed to get device count");

    info!("Number of devices: {:?}", count);

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
        info!("No devices.. exiting");
        return;
    }

    info!("Printing out device properties...");

    for device in &all_devices {
        //Devices have 3 properties

        //The device info, which is static information about the device
        info!("device info: {:?}", device.device_info);

        //The device index (this is not guaranteed to be static unfortunately, as devices can be plugged/unplugged
        // (see the events example on how to handle this)
        info!("device index is: {:?}", device.device_index);

        //And the leds vector, which contains all the leds all the device
        //These leds can be individually used, or used "through" the device
        info!("device leds: {:?}", device.leds)
    }
}
