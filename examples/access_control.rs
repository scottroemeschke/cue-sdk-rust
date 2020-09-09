extern crate cue_sdk;
extern crate env_logger;

use env_logger::Env;
use log::info;

pub fn main() {
    env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));
    let mut sdk = cue_sdk::initialize().expect("failed to initialize sdk");
    info!("Requesting exclusive access control...");
    sdk.request_exclusive_access_control()
        .expect("failed to get exclusive access control");

    //do some stuff with exclusive access control (no other clients can make calls during this time)

    info!("Received exclusive access control, do your thing!");
    // code code code ...

    info!("Attempting to release exclusive access control...");
    //maybe you decide to be nice and let other kids have a turn
    sdk.release_exclusive_access_control()
        .expect("failed to release exclusive access control.");

    //if you don't release, them dropping the sdk will perform a release of exclusive access if needed

    drop(sdk); //the sdk will release exclusive access control

    info!("The SDK was dropped, so if the exclusive access control wasn't manually released earlier, it would have been released.");
}
