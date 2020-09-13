#[cfg(feature = "async")]
use tokio::sync::mpsc;
use crate::sdk::{CueSdkClient, UnsubscribeFromEventsError};
use failure::_core::sync::atomic::AtomicBool;
use super::CueEvent;
use crate::errors::CueSdkErrorResult;
use crate::event::CueEventFromFfiError;

#[cfg(feature = "async")]
pub struct EventSubscription<'sdk_client> {
    channel: mpsc::Receiver<Result<CueEvent, CueEventFromFfiError>>,
    sdk: &'sdk_client CueSdkClient,
}

#[cfg(feature = "async")]
impl <'sdk_client> EventSubscription<'sdk_client> {
    pub(crate) fn new(rx: mpsc::Receiver<Result<CueEvent, CueEventFromFfiError>>, sdk_client: &'sdk_client CueSdkClient) -> Self {
        EventSubscription {
            channel: rx,
            sdk: sdk_client,
        }
    }

    pub async fn next_event(&mut self) -> Option<Result<CueEvent, CueEventFromFfiError>> {
        self.channel.recv().await
    }

    pub fn try_next_event(&mut self) -> Option<Result<CueEvent, CueEventFromFfiError>> {
        self.channel.try_recv()
            .ok()
    }

    pub fn unsubscribe(mut self) -> Result<(), UnsubscribeFromEventsError> {
        self.channel.close();
        self.sdk.unsubscribe_from_events()
    }
}

#[cfg(feature = "async")]
impl <'sdk_client> Drop for EventSubscription<'sdk_client> {
    fn drop(&mut self) {
        self.channel.close();
    }
}
