extern crate cue_sdk;
extern crate env_logger;

use env_logger::Env;
use log::{info, warn};

use cue_sdk::event::CueEvent;
use std::time::Duration;

pub fn main() {
    env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    rt.block_on(async {
        let mut sdk = cue_sdk::initialize().expect("failed to initialize sdk");

        info!("subscribing to events..");
        sdk.subscribe_for_events(|result| match result {
            Ok(ev) => match ev {
                CueEvent::KeyEvent(device_id, key_id, is_pressed) => {
                    info!("A key event occurred!");
                    info!(
                        "device_id: {:?}, key_id: {:?}, is now pressed: {:?}",
                        device_id, key_id, is_pressed
                    );
                }
                CueEvent::DeviceConnectedStatusChangedEvent(device_id, is_connected) => {
                    info!("A device connection status changed event occurred!");
                    info!(
                        "device_id: {:?}, is now connected: {:?}",
                        device_id, is_connected
                    );
                }
            },
            Err(err) => {
                warn!("There was an error getting an event from ffi: {:?}", err);
            }
        })
        .expect("failed to subscribe for events.");

        info!("We have subscribed... waiting 5 seconds...");
        let handle = task::sleep(Duration::from_secs(5));
        handle.await;
    });

    //We will unsubscribe for events if you are currently subscribed and drop the SDK
    info!("sdk has been dropped so we have unsubscribed from events");
}
