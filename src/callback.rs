//! Internal callback trampolines for the iCUE SDK.
//!
//! Most SDK callbacks follow the same pattern:
//! 1. A `Pin<Box<Sender<T>>>` is heap-allocated and its raw pointer passed as the
//!    `context` parameter to the SDK.
//! 2. A bare `extern "C" fn` trampoline casts the context back and sends the data
//!    through the channel.
//! 3. The owning struct (`EventSubscription`) keeps the `Pin<Box<…>>`
//!    alive for exactly as long as the SDK holds the pointer.
//!
//! **Session state** is the exception: its sender lives in a process-wide static
//! so the SDK's background thread can never dereference a freed pointer (see
//! issue #18).

use std::pin::Pin;
use std::sync::mpsc;
use std::sync::Mutex;

use core::ffi::c_void;
use cue_sdk_sys as ffi;

use crate::event::Event;

// ---- Session state callback ------------------------------------------------

/// Data sent through the session-state channel.
#[derive(Debug, Clone)]
pub(crate) struct SessionStateChange {
    pub state: ffi::CorsairSessionState,
    pub details: ffi::CorsairSessionDetails,
}

/// Process-wide sender for session state changes.
///
/// The trampoline reads from this static instead of dereferencing a `context`
/// pointer, so the pointer is always valid even if the SDK's background thread
/// fires the callback during or after `CorsairDisconnect`.
static SESSION_STATE_TX: Mutex<Option<mpsc::Sender<SessionStateChange>>> = Mutex::new(None);

/// Install a sender for session state changes into the process-wide static.
pub(crate) fn install_session_sender(tx: mpsc::Sender<SessionStateChange>) {
    // SAFETY (logical): Any previous sender is dropped here.  This is fine
    // because it means the old receiver will see a disconnected channel.
    *SESSION_STATE_TX.lock().unwrap() = Some(tx);
}

/// Remove the session state sender, making the trampoline a no-op.
///
/// Must be called **before** `CorsairDisconnect` so the SDK's background
/// thread cannot send into a half-dropped channel.
pub(crate) fn clear_session_sender() {
    *SESSION_STATE_TX.lock().unwrap() = None;
}

/// Return a raw pointer suitable for the SDK `context` parameter.
pub(crate) fn sender_as_context<T>(sender: &Pin<Box<mpsc::Sender<T>>>) -> *mut c_void {
    let ptr: *const mpsc::Sender<T> = &**sender;
    ptr as *mut c_void
}

/// `extern "C"` trampoline for `CorsairSessionStateChangedHandler`.
///
/// # Safety
///
/// - `event_data` must point to a valid `CorsairSessionStateChanged`
///   (guaranteed by the SDK contract).
/// - The `context` parameter is ignored; the sender is read from the
///   process-wide `SESSION_STATE_TX` static.
pub(crate) unsafe extern "C" fn session_state_trampoline(
    _context: *mut c_void,
    event_data: *const ffi::CorsairSessionStateChanged,
) {
    // SAFETY: `event_data` is provided by the SDK and valid for the duration of
    // this callback invocation.
    let data = unsafe { &*event_data };
    if let Ok(guard) = SESSION_STATE_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(SessionStateChange {
                state: data.state,
                details: data.details,
            });
        }
    }
}

// ---- Event callback --------------------------------------------------------

/// Pinned sender kept alive by `EventSubscription`.
pub(crate) type EventSender = Pin<Box<mpsc::Sender<Event>>>;

/// Create a (sender, receiver) pair for SDK events.
pub(crate) fn event_channel() -> (EventSender, mpsc::Receiver<Event>) {
    let (tx, rx) = mpsc::channel();
    (Box::pin(tx), rx)
}

/// `extern "C"` trampoline for `CorsairEventHandler`.
///
/// # Safety
///
/// - `context` must be a valid pointer to a live `mpsc::Sender<Event>`
///   (guaranteed by the `Pin<Box<Sender>>` kept alive in `EventSubscription`).
/// - `event` must point to a valid `CorsairEvent` (guaranteed by the SDK).
pub(crate) unsafe extern "C" fn event_trampoline(
    context: *mut c_void,
    event: *const ffi::CorsairEvent,
) {
    // SAFETY: `context` was created by `sender_as_context` from a pinned boxed
    // sender that outlives the SDK event subscription.
    let tx = unsafe { &*(context as *const mpsc::Sender<Event>) };
    // SAFETY: `event` is provided by the SDK and valid for the duration of this
    // callback invocation.
    let ev = unsafe { &*event };
    if let Some(parsed) = Event::from_ffi(ev) {
        let _ = tx.send(parsed);
    }
}

// ---- Async flush callback --------------------------------------------------

/// Pinned sender for a one-shot flush result.
pub(crate) type FlushSender = Pin<Box<mpsc::Sender<ffi::CorsairError>>>;

/// Create a (sender, receiver) pair for an async flush result.
pub(crate) fn flush_channel() -> (FlushSender, mpsc::Receiver<ffi::CorsairError>) {
    let (tx, rx) = mpsc::channel();
    (Box::pin(tx), rx)
}

/// `extern "C"` trampoline for `CorsairAsyncCallback`.
///
/// # Safety
///
/// - `context` must be a valid pointer to a live `mpsc::Sender<CorsairError>`
///   (guaranteed by the `Pin<Box<Sender>>` kept alive in `flush_led_colors`).
pub(crate) unsafe extern "C" fn flush_trampoline(context: *mut c_void, error: ffi::CorsairError) {
    // SAFETY: `context` was created by `sender_as_context` from a pinned boxed
    // sender that is held on the stack in `Session::flush_led_colors` until the
    // receiver gets the result.
    let tx = unsafe { &*(context as *const mpsc::Sender<ffi::CorsairError>) };
    let _ = tx.send(error);
}

// ---- Async (tokio) variants -----------------------------------------------

#[cfg(feature = "async")]
mod async_impl {
    use std::pin::Pin;

    use core::ffi::c_void;
    use cue_sdk_sys as ffi;
    use tokio::sync::mpsc as tokio_mpsc;

    use crate::event::Event;

    /// Pinned sender kept alive by `AsyncEventSubscription`.
    pub(crate) type AsyncEventSender = Pin<Box<tokio_mpsc::UnboundedSender<Event>>>;

    /// Pinned sender for a one-shot async flush result.
    pub(crate) type AsyncFlushSender = Pin<Box<tokio_mpsc::UnboundedSender<ffi::CorsairError>>>;

    /// Create a (sender, receiver) pair for async SDK events.
    pub(crate) fn async_event_channel() -> (AsyncEventSender, tokio_mpsc::UnboundedReceiver<Event>)
    {
        let (tx, rx) = tokio_mpsc::unbounded_channel();
        (Box::pin(tx), rx)
    }

    /// Create a (sender, receiver) pair for an async flush result.
    pub(crate) fn async_flush_channel() -> (
        AsyncFlushSender,
        tokio_mpsc::UnboundedReceiver<ffi::CorsairError>,
    ) {
        let (tx, rx) = tokio_mpsc::unbounded_channel();
        (Box::pin(tx), rx)
    }

    /// Return a raw pointer suitable for the SDK `context` parameter.
    pub(crate) fn async_sender_as_context<T>(
        sender: &Pin<Box<tokio_mpsc::UnboundedSender<T>>>,
    ) -> *mut c_void {
        let ptr: *const tokio_mpsc::UnboundedSender<T> = &**sender;
        ptr as *mut c_void
    }

    /// `extern "C"` trampoline for `CorsairEventHandler` (async variant).
    ///
    /// # Safety
    ///
    /// - `context` must be a valid pointer to a live
    ///   `tokio_mpsc::UnboundedSender<Event>` (guaranteed by the
    ///   `Pin<Box<Sender>>` kept alive in `AsyncEventSubscription`).
    /// - `event` must point to a valid `CorsairEvent` (guaranteed by the SDK).
    pub(crate) unsafe extern "C" fn async_event_trampoline(
        context: *mut c_void,
        event: *const ffi::CorsairEvent,
    ) {
        // SAFETY: `context` was created by `async_sender_as_context` from a
        // pinned boxed sender that outlives the SDK event subscription.
        let tx = unsafe { &*(context as *const tokio_mpsc::UnboundedSender<Event>) };
        // SAFETY: `event` is provided by the SDK and valid for the duration of
        // this callback invocation.
        let ev = unsafe { &*event };
        if let Some(parsed) = Event::from_ffi(ev) {
            let _ = tx.send(parsed);
        }
    }

    /// `extern "C"` trampoline for `CorsairAsyncCallback` (async variant).
    ///
    /// # Safety
    ///
    /// - `context` must be a valid pointer to a live
    ///   `tokio_mpsc::UnboundedSender<CorsairError>` (guaranteed by the
    ///   `Pin<Box<Sender>>` kept alive in `flush_led_colors_async`).
    pub(crate) unsafe extern "C" fn async_flush_trampoline(
        context: *mut c_void,
        error: ffi::CorsairError,
    ) {
        // SAFETY: `context` was created by `async_sender_as_context` from a
        // pinned boxed sender held by the async method until the receiver
        // gets the result.
        let tx = unsafe { &*(context as *const tokio_mpsc::UnboundedSender<ffi::CorsairError>) };
        let _ = tx.send(error);
    }
}

#[cfg(feature = "async")]
pub(crate) use async_impl::*;
