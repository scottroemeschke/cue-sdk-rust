extern crate cue_sdk;
extern crate env_logger;

use cue_sdk::led::LedColor;
use env_logger::Env;
use failure::_core::time::Duration;
use log::info;
use rand::Rng;

pub fn main() {
    env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));
    let sdk = cue_sdk::initialize().expect("failed to initialize sdk");

    info!("Getting all devices...");

    let mut all_devices = sdk.get_all_devices().expect("failed to get all devices");

    if all_devices.is_empty() {
        info!("No devices.. exiting");
        return;
    }

    info!("Setting all the leds to random in the CUESDK color buffer");

    //Updating LED colors requires calling color updating functions on the sdk (or led struct themselves)
    //followed by flushing the "color buffer" in the sdk
    for device in &mut all_devices {
        // we could call sdk.update_leds_color_buffer(), but let's just do every led
        // in a real app we could do this step in parallel but let's just do this simply now
        for led in &mut device.leds {
            led.update_color_buffer(random_led_color())
                .expect("failed to update color buffer");
        }
    }

    info!("Flushing the buffer now!");

    //You can flush the buffer synchronously if you don't mind waiting (no GUI etc)
    sdk.flush_led_colors_update_buffer_sync()
        .expect("failed to flush the color buffer");

    //You can also flush asynchronously with a closure callback

    /*

    sdk.flush_led_colors_update_buffer(|result| {
        if Err(e) = result {
            warn!("failed to flush the color buffer.. received from closure");
        }
    }).expect("failed to trigger the async color buffer flush with closure callback");

     */

    info!("Sleeping for a few seconds!");
    std::thread::sleep(Duration::from_secs(5));

    info!("Done!");
}

fn random_led_color() -> LedColor {
    let mut rng = rand::thread_rng();
    LedColor {
        red: rng.gen(),
        green: rng.gen(),
        blue: rng.gen(),
    }
}
