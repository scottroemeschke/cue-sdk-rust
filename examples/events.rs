use std::time::Duration;

use cue_sdk::event::Event;

fn main() {
    let session = cue_sdk::connect().expect("failed to connect");
    let _details = session
        .wait_for_connection(Duration::from_secs(5))
        .expect("timeout waiting for iCUE");

    let subscription = session.subscribe_for_events().expect("failed to subscribe");

    println!("Listening for events... Press Ctrl+C to exit.");

    for event in subscription.iter() {
        match event {
            Event::DeviceConnectionChanged {
                device_id,
                is_connected,
            } => {
                let action = if is_connected {
                    "connected"
                } else {
                    "disconnected"
                };
                println!("Device {} {}", device_id, action);
            }
            Event::KeyEvent {
                device_id,
                key_id,
                is_pressed,
            } => {
                let action = if is_pressed { "pressed" } else { "released" };
                println!("Key {:?} {} on device {}", key_id, action, device_id);
            }
        }
    }
}
