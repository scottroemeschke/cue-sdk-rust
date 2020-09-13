#[cfg(feature = "async")]
use tokio::sync::mpsc;

use super::{CueEvent, CueEventFromFfiError};
use crate::sdk::{CueSdkClient, UnsubscribeFromEventsError};

/// An event subscription, which you can await for the next event, optimistically "check"
/// synchronously for the next event, and unsubscribe.
///
/// Note: Dropping this will cause also unsubscribe from events.
#[cfg(feature = "async")]
pub struct EventSubscription<'sdk_client> {
    channel: mpsc::Receiver<Result<CueEvent, CueEventFromFfiError>>,
    sdk: &'sdk_client CueSdkClient,
}

#[cfg(feature = "async")]
impl<'sdk_client> EventSubscription<'sdk_client> {
    pub(crate) fn new(
        rx: mpsc::Receiver<Result<CueEvent, CueEventFromFfiError>>,
        sdk_client: &'sdk_client CueSdkClient,
    ) -> Self {
        EventSubscription {
            channel: rx,
            sdk: sdk_client,
        }
    }

    /// Returns a future with the next event (or error).
    pub async fn next_event(&mut self) -> Option<Result<CueEvent, CueEventFromFfiError>> {
        self.channel.recv().await
    }

    /// Immediately returns an option with the next event (or error).
    pub fn try_next_event(&mut self) -> Option<Result<CueEvent, CueEventFromFfiError>> {
        self.channel.try_recv().ok()
    }

    /// Unsubscribes from events from the iCUE SDK and consumes this struct.
    ///
    /// Note: Dropping the `EventSubscription` will also unsubscribe.
    pub fn unsubscribe(mut self) -> Result<(), UnsubscribeFromEventsError> {
        self.channel.close();
        self.sdk.unsubscribe_from_events()
    }
}

#[cfg(feature = "async")]
impl<'sdk_client> Drop for EventSubscription<'sdk_client> {
    fn drop(&mut self) {
        self.channel.close();
        let _ = self.sdk.unsubscribe_from_events();
    }
}
