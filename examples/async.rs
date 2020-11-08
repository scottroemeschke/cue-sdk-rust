extern crate cue_sdk;
extern crate env_logger;

use cue_sdk::sdk::GetAllDevicesError;
use env_logger::Env;
use log::info;

use cue_sdk::device::{CueDevice, DeviceType};
use cue_sdk::event::{CueEvent, CueEventFromFfiError};
use cue_sdk::led::LedColor;
use cue_sdk::property::{BooleanPropertyKey, Int32PropertyKey};

pub fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    rt.block_on(|| async {
        env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));
        let sdk = cue_sdk::initialize().expect("failed to initialize sdk");

        // There is an optional "async" feature that changes various callback/closure based methods
        // to be async/await based instead.

        //Let's write to the color buffer and then flush async."

        info!("Getting all devices");

        let mut devices = sdk.get_all_devices().expect("failed to get all devices");

        if devices.len() == 0 {
            warn!("No devices found.")
        }

        let device = devices.first_mut().unwrap();

        for led in device.leds.iter_mut() {
            led.update_color_buffer(LedColor {
                red: 20,
                blue: 200,
                green: 100,
            })
            .expect("There was an error updating led color");
        }

        sdk.flush_led_colors_update_buffer_async()
            .await
            .expect("failed to flush led colors buffer async");

        //Let's now subscribe to events and await on a couple.

        let mut subscription = sdk
            .subscribe_for_events_async()
            .expect("failed to subscribe to events async");

        match subscription.try_next_event() {
            Some(ev) => {
                info!(
                    "There was immediately an event. \
            That's very surprising, perhaps the iCUE SDK did something odd.\
            The event: {:?}",
                    ev
                );
            }
            None => {
                info!(
                    "No event when we checked immediately after subscribing, \
            but we can await for one."
                );
            }
        }

        info!("Awaiting the next event...");

        let ev = subscription.next_event().await;

        info!("There was an event: {:?}", ev);

        let ev2 = subscription.next_event().await;

        info!("There was another event: {:?}", ev2);

        //Let's unsubscribe now, we can do that manually through unsubscribe, or
        //we when the subscription is dropped, you will automatically be unsubscribed.
        drop(subscription)
    });
}
