use std::time::Duration;

use cue_sdk::device::DeviceType;
use cue_sdk::led::LedColor;

fn main() {
    let session = cue_sdk::connect().expect("failed to connect");
    let _details = session
        .wait_for_connection(Duration::from_secs(5))
        .expect("timeout waiting for iCUE");

    let devices = session
        .get_devices(DeviceType::KEYBOARD)
        .expect("failed to get devices");

    let device = devices.first().expect("no keyboard found");
    println!("Using keyboard: {}", device.model);

    // Get all LED positions and set them to red.
    let positions = session
        .get_led_positions(&device.id)
        .expect("failed to get LED positions");

    let colors: Vec<LedColor> = positions
        .iter()
        .map(|pos| LedColor::rgb(pos.id, 255, 0, 0))
        .collect();

    session
        .set_led_colors(&device.id, &colors)
        .expect("failed to set colors");

    println!("Set {} LEDs to red. Press Ctrl+C to exit.", colors.len());

    // Keep running so the colors stay visible.
    std::thread::sleep(Duration::from_secs(10));
}
