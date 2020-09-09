extern crate cue_sdk;
extern crate env_logger;

use cue_sdk::device::{CueDevice, DeviceType};
use cue_sdk::led::{CueLed, LedColor};

use env_logger::Env;
use log::info;
use std::thread::sleep;
use std::time::Duration;

pub fn main() {
    env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));
    let sdk = cue_sdk::initialize().expect("failed to initialize sdk");

    info!("Getting all devices...");

    let mut all_devices = sdk.get_all_devices().expect("failed to get all devices");

    if all_devices.is_empty() {
        info!("No devices.. exiting");
        return;
    }

    info!("Making all of the keys red.. for some reason.");

    //Let's filter the list of devices down to just keyboards
    let mut filtered_devices = all_devices
        .iter_mut()
        .filter(|d| d.device_info.device_type == Some(DeviceType::Keyboard))
        .collect::<Vec<&mut CueDevice>>();

    info!(
        "Number of keyboards currently connected: {:?}",
        filtered_devices.len()
    );

    if filtered_devices.len() > 1 {
        info!("Wait, you have more than one keyboard connected... uh... I guess you're a {:?}x engineer??", filtered_devices.len());
    }

    let red = LedColor {
        red: 255,
        green: 0,
        blue: 0,
    };

    //Now let's just get references to all of the leds in those keyboards
    let mut leds = filtered_devices
        .iter_mut()
        .map(|d| &mut d.leds)
        .flatten()
        .map(|led| (&red, led))
        .collect::<Vec<(&LedColor, &mut CueLed)>>();

    info!("Updating color buffer with all keyboard leds set to red");

    sdk.update_leds_color_buffer(leds.as_mut_slice())
        .expect("failed to update led colors buffer");

    info!("And flushing the colors buffer...");

    //You can also flush asynchronously with a closure callback, or async returning a Future.
    sdk.flush_led_colors_update_buffer_sync()
        .expect("failed to flush the colors buffer");

    info!("All the leds should be red. Yay?");

    sleep(Duration::from_secs(5));

    info!("Ending now.");
}
