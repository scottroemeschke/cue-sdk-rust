use std::time::Duration;

use cue_sdk::device::DeviceType;

fn main() {
    let session = cue_sdk::connect().expect("failed to connect");
    let _details = session
        .wait_for_connection(Duration::from_secs(5))
        .expect("timeout waiting for iCUE");

    let devices = session
        .get_devices(DeviceType::ALL)
        .expect("failed to get devices");

    println!("Found {} device(s):", devices.len());
    for dev in &devices {
        println!(
            "  {} (serial: {}, type: {:?}, LEDs: {}, channels: {})",
            dev.model, dev.serial, dev.device_type, dev.led_count, dev.channel_count
        );
    }
}
