use std::time::Duration;

fn main() {
    let session = cue_sdk::connect().expect("failed to connect");
    let details = session
        .wait_for_connection(Duration::from_secs(5))
        .expect("timeout waiting for iCUE");

    println!("Client version: {}", details.client_version);
    println!("Server version: {}", details.server_version);
    println!("Host version:   {}", details.server_host_version);
}
