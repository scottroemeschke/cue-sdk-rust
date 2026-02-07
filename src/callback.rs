//! Internal callback trampolines for the iCUE SDK.
//!
//! Every SDK callback follows the same pattern:
//! 1. A `Pin<Box<Sender<T>>>` is heap-allocated and its raw pointer passed as the
//!    `context` parameter to the SDK.
//! 2. A bare `extern "C" fn` trampoline casts the context back and sends the data
//!    through the channel.
//! 3. The owning struct (`Session`, `EventSubscription`) keeps the `Pin<Box<…>>`
//!    alive for exactly as long as the SDK holds the pointer.

use std::pin::Pin;
use std::sync::mpsc;

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

/// Pinned sender kept alive by `Session`.
pub(crate) type SessionStateSender = Pin<Box<mpsc::Sender<SessionStateChange>>>;

/// Create a (sender, receiver) pair for session state changes.  The sender is
/// pinned on the heap so its address is stable for the SDK callback.
pub(crate) fn session_state_channel() -> (SessionStateSender, mpsc::Receiver<SessionStateChange>) {
    let (tx, rx) = mpsc::channel();
    (Box::pin(tx), rx)
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
/// - `context` must be a valid pointer to a live `mpsc::Sender<SessionStateChange>`
///   (guaranteed by the `Pin<Box<Sender>>` kept alive in `Session`).
/// - `event_data` must point to a valid `CorsairSessionStateChanged`
///   (guaranteed by the SDK contract).
pub(crate) unsafe extern "C" fn session_state_trampoline(
    context: *mut c_void,
    event_data: *const ffi::CorsairSessionStateChanged,
) {
    // SAFETY: `context` was created by `sender_as_context` from a pinned boxed
    // sender that outlives the SDK callback registration.
    let tx = unsafe { &*(context as *const mpsc::Sender<SessionStateChange>) };
    // SAFETY: `event_data` is provided by the SDK and valid for the duration of
    // this callback invocation.
    let data = unsafe { &*event_data };
    let _ = tx.send(SessionStateChange {
        state: data.state,
        details: data.details,
    });
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
