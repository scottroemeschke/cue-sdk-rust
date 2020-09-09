extern crate cue_sdk;
extern crate env_logger;

use cue_sdk::initialization::HandshakeError;
use env_logger::Env;
use log::info;

fn main() {
    env_logger::init_from_env(Env::new().filter("CUE_SDK_EXAMPLES_LOG_LEVEL"));
    info!("Performing SDK initialization...");

    let sdk_client = cue_sdk::initialize()
        .map_err(|e| {
            match &e {
                HandshakeError::InitialHandshakeError(_) => {
                    //this initial handshake error has a "CueSdkError"
                }
                HandshakeError::ProtocolDetailsError(_) => {
                    // this error mean the protocol details struct returned from the SDK didn't
                    // meet some expected invariant
                }
            };
            e
        })
        .expect("Failed to initialize cue sdk");

    info!("SDK initialization complete.");
    info!("SDK protocol details: {:?}", sdk_client.protocol_details);
    info!("Initial layer priority: {:?}", sdk_client.priority);

    // It's highly recommended to use a single SDK instance.
    // Go on to do things with the SDK ...
}
