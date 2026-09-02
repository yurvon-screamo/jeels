// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

//! macOS implementation of `start_auth` via `ASWebAuthenticationSession`.
//!
//! Mirrors the iOS Swift plugin semantics: the OAuth URL opens in a dedicated
//! authentication browser session, and the intercepted `origa://` callback
//! URL resolves the command promise. User cancellation resolves as
//! `"cancelled"` so the frontend can treat both platforms identically.
//!
//! Threading: `ASWebAuthenticationSession` must be created and started on the
//! main thread — `commands.rs` dispatches here through
//! `AppHandle::run_on_main_thread` and awaits an mpsc channel. The
//! completion handler, however, is NOT guaranteed to arrive on the main
//! queue: macOS 26 has been observed invoking it on a background queue after
//! a redirect-initiated interception, so the handler must be sound on any
//! thread. It only performs thread-safe work: owned-string extraction from
//! the immutable `NSURL`/`NSError` arguments, an mpsc send, and a
//! main-queue dispatch hop that breaks the session ↔ block retain cycle so
//! the ObjC dealloc cascade stays on the main thread.

use std::sync::{Arc, Mutex};

use block2::RcBlock;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::NSWindow;
use objc2_authentication_services::{
    ASWebAuthenticationPresentationContextProviding, ASWebAuthenticationSession,
    ASWebAuthenticationSessionErrorCode,
};
use objc2_foundation::{MainThreadMarker, NSError, NSObjectProtocol, NSString, NSURL};
use tauri::Manager;

use crate::commands::AuthResult;

/// Type alias used by the generated protocol for the presentation anchor.
/// The crate keeps it as `NSObject`; we store an `NSWindow` in it.
type PresentationAnchor = NSObject;

/// Minimal libdispatch FFI. Routing the bundle drop onto the main queue
/// needs only these two symbols, which libSystem provides in every macOS
/// process — no objc2-dispatch dependency required.
mod dispatch {
    use std::ffi::c_void;

    use block2::RcBlock;

    /// Opaque `dispatch_queue_t`. The ABI passes the queue object as a
    /// single pointer; the main queue is a global singleton that needs no
    /// retain/release.
    #[repr(transparent)]
    pub(crate) struct MainQueue(*mut c_void);

    unsafe extern "C" {
        fn dispatch_get_main_queue() -> MainQueue;
        fn dispatch_async(queue: MainQueue, block: *mut c_void);
    }

    /// Runs `f` on the main dispatch queue.
    pub(crate) fn exec_on_main<F>(f: F)
    where
        F: block2::IntoBlock<'static, (), ()>,
    {
        let block = RcBlock::new(f);
        // SAFETY: both symbols resolve in libSystem; the main queue is the
        // global main queue. `dispatch_async` copies (retains) the block
        // before returning, so dropping our `RcBlock` afterwards only
        // releases our reference — the queue keeps its own until the block
        // has run, and the block then deallocates on the main thread.
        unsafe {
            dispatch_async(
                dispatch_get_main_queue(),
                RcBlock::as_ptr(&block) as *mut c_void,
            );
        }
    }
}

/// Session + provider kept alive by the completion block (see the slot
/// comment in `start_session`).
type SessionBundle = (
    Retained<ASWebAuthenticationSession>,
    Retained<AuthAnchorProvider>,
);

/// The slot is shared with the completion block, which macOS 26 may invoke
/// on a background queue: `Arc` clones are thread-safe, and `Mutex` guards
/// the single take.
type SessionSlot = Arc<Mutex<Option<SendableBundle>>>;

/// `Send` wrapper required to store [`SessionBundle`] in a [`SessionSlot`]:
/// `Retained` handles are `!Send` because their classes may have
/// main-thread-only methods.
///
/// # SAFETY
/// `Send` only permits *storing and moving* the bundle alongside the
/// completion block, which macOS 26 may invoke on a background queue. The
/// final release — which runs `-dealloc` on the dropping thread — never
/// happens off the main thread: `drop_slot_contents` is called exclusively
/// from main-thread contexts (the start-failure path and the completion's
/// main-queue dispatch hop). This matters because the bundle carries AppKit
/// objects — the anchor provider's `NSWindow` ivar must deallocate on the
/// main thread, and the `AnyThread` marker on `ASWebAuthenticationSession`
/// is a silent `NSObject` default rather than a documented dealloc
/// guarantee.
struct SendableBundle(SessionBundle);
unsafe impl Send for SendableBundle {}

/// Stores the bundle into the slot, tolerating a poisoned mutex: the
/// critical sections guarded by this lock are a plain assignment and a
/// `take`, neither of which can panic, so a poisoned lock carries no
/// broken invariant and the payload is still stored.
fn fill_slot(slot: &SessionSlot, bundle: SendableBundle) {
    *slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(bundle);
}

/// Takes the bundle out of the slot with the same poison tolerance as
/// [`fill_slot`]: the payload is handed out exactly once.
fn take_slot(slot: &SessionSlot) -> Option<SendableBundle> {
    slot.lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
}

/// Drops the bundle taken from the slot. Must only be called on the main
/// thread (see `SendableBundle`): the retain cycle is broken here and the
/// dealloc cascade runs inline.
fn drop_slot_contents(slot: &SessionSlot) {
    if let Some(SendableBundle(bundle)) = take_slot(slot) {
        drop(bundle);
    }
}

define_class!(
    // SAFETY:
    // - Superclass `NSObject` has no subclassing requirements.
    // - The type does not implement `Drop`.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = Retained<PresentationAnchor>]
    struct AuthAnchorProvider;

    // SAFETY: `NSObjectProtocol` has no safety requirements.
    unsafe impl NSObjectProtocol for AuthAnchorProvider {}

    // SAFETY: The method signature matches the generated protocol declaration.
    unsafe impl ASWebAuthenticationPresentationContextProviding for AuthAnchorProvider {
        // Retained-returning protocol methods are registered via `method_id`.
        #[unsafe(method_id(presentationAnchorForWebAuthenticationSession:))]
        unsafe fn presentation_anchor(
            &self,
            _session: &ASWebAuthenticationSession,
        ) -> Retained<PresentationAnchor> {
            self.ivars().clone()
        }
    }
);

impl AuthAnchorProvider {
    fn new(window: Retained<NSWindow>, mtm: MainThreadMarker) -> Retained<Self> {
        // NSWindow → NSResponder → NSObject: two hops up the superclass chain.
        let anchor: Retained<PresentationAnchor> = window.into_super().into_super();
        let this = Self::alloc(mtm).set_ivars(anchor);
        // SAFETY: The signature of `NSObject`'s `init` is correct.
        unsafe { msg_send![super(this), init] }
    }
}

/// Creates and starts an auth session on the main thread. Guarantees exactly
/// one send through `tx`: from the completion handler, or synchronously with
/// the real failure cause when the session cannot be created or started.
pub(crate) fn start_session<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    url: &str,
    callback_scheme: &str,
    tx: std::sync::mpsc::Sender<Result<AuthResult, String>>,
) {
    // Every early-exit path owns `tx` and sends the failure itself, so the
    // frontend always learns the real cause.
    let Some(mtm) = MainThreadMarker::new() else {
        let _ = tx.send(Err("start_session must run on the main thread".to_string()));
        return;
    };

    // Apple only loads http(s) auth pages; reject anything else early so a
    // compromised renderer cannot aim the session at custom schemes. Scheme
    // comparison is case-insensitive per RFC 3986.
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        let _ = tx.send(Err(format!(
            "rejected non-http(s) authentication URL: {url}"
        )));
        return;
    }

    let Some(webview_window) = app.get_webview_window("main") else {
        let _ = tx.send(Err("no main window".to_string()));
        return;
    };
    let ns_window_ptr = match webview_window.ns_window() {
        Ok(ptr) => ptr,
        Err(e) => {
            let _ = tx.send(Err(format!("failed to get the main NSWindow: {e}")));
            return;
        },
    };
    // SAFETY: Tauri returns a valid `NSWindow` pointer for the main window on
    // macOS; `Retained::retain` balances its +1 with our eventual release.
    let retained_window = unsafe { Retained::retain(ns_window_ptr.cast()) };
    let Some(window) = retained_window else {
        let _ = tx.send(Err("failed to retain the main NSWindow".to_string()));
        return;
    };

    let provider = AuthAnchorProvider::new(window, mtm);
    let proto = ProtocolObject::from_ref(&*provider);

    let url_string = NSString::from_str(url);
    let Some(ns_url) = NSURL::initWithString(NSURL::alloc(), &url_string) else {
        let _ = tx.send(Err(format!("invalid authentication URL: {url}")));
        return;
    };
    let scheme = NSString::from_str(callback_scheme);

    // The slot is owned by the completion block. Filling it after creation
    // (but before `start`) keeps the session and provider alive for as long
    // as the system holds the block; taking them at completion time breaks
    // the session ↔ block retain cycle so everything deallocates cleanly.
    let bundle_slot: SessionSlot = Arc::new(Mutex::new(None));

    let tx_for_completion = tx.clone();
    let bundle_for_completion = Arc::clone(&bundle_slot);
    let completion = RcBlock::new(move |callback_url: *mut NSURL, error: *mut NSError| {
        // The pointer arguments are only valid for the duration of this
        // call, so owned values are extracted first. `NSURL` and `NSError`
        // are immutable: reading `absoluteString`/`localizedDescription`
        // from any thread is safe (macOS 26 may invoke this handler off the
        // main queue — observed on a redirect-initiated interception).
        let result = autoreleasepool(|_pool| {
            if let Some(error) = as_ref(error) {
                return Err(describe_error(error));
            }
            let Some(callback_url) = as_ref(callback_url) else {
                return Err("completion returned neither URL nor error".to_string());
            };
            match callback_url.absoluteString() {
                Some(string) => Ok(AuthResult {
                    url: string.to_string(),
                }),
                None => Err("callback URL has no absolute string".to_string()),
            }
        });

        // Blocks are `Fn` (callable repeatedly by contract), so clone the
        // sender per invocation instead of moving it out of the closure.
        let _ = tx_for_completion.clone().send(result);

        // Break the session ↔ block retain cycle now that the system is
        // done with both — on the main thread, because the final release
        // runs `-dealloc` on the dropping thread and the bundle holds
        // AppKit objects (see `SendableBundle`).
        if MainThreadMarker::new().is_some() {
            drop_slot_contents(&bundle_for_completion);
        } else {
            let slot_for_drop = Arc::clone(&bundle_for_completion);
            dispatch::exec_on_main(move || drop_slot_contents(&slot_for_drop));
        }
    });

    let session = unsafe {
        // SAFETY: The completion handler is a valid block pointer.
        //
        // `initWithURL:callbackURLScheme:completionHandler:` is deprecated in
        // favour of the `ASWebAuthenticationSessionCallback` variant, but the
        // replacement requires pre-building a callback object and changes the
        // interception model; the deprecated initializer remains fully
        // functional for custom-scheme flows.
        #[allow(deprecated)]
        ASWebAuthenticationSession::initWithURL_callbackURLScheme_completionHandler(
            ASWebAuthenticationSession::alloc(),
            &ns_url,
            Some(&scheme),
            RcBlock::as_ptr(&completion),
        )
    };

    // Weak property on the session side — our strong reference lives in the
    // bundle slot, so the provider outlives the presentation window request.
    unsafe { session.setPresentationContextProvider(Some(proto)) };
    // Share the user's existing Safari login between sessions (Apple default).
    unsafe { session.setPrefersEphemeralWebBrowserSession(false) };

    fill_slot(&bundle_slot, SendableBundle((session.clone(), provider)));

    // SAFETY: The session was created above; `start` has no additional safety
    // requirements beyond a valid receiver.
    if !unsafe { session.start() } {
        // No completion will fire — drop the bundle so the session and the
        // system's copy of the block deallocate instead of leaking. This
        // runs on the main thread, so the dealloc is inline.
        drop_slot_contents(&bundle_slot);
        tracing::warn!("[aswebauth] ASWebAuthenticationSession::start returned false");
        let _ = tx.send(Err("failed to start the authentication session".to_string()));
    }
}

fn as_ref<T>(ptr: *mut T) -> Option<&'static T> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: Callers pass pointers handed over by the ObjC runtime;
        // they are valid for the duration of the completion handler.
        Some(unsafe { &*ptr })
    }
}

/// Maps an ObjC `NSError` from the completion handler onto the frontend
/// contract. User dismissal becomes `"cancelled"` (same as iOS); everything
/// else surfaces its localized description.
fn describe_error(error: &NSError) -> String {
    if error.code() == ASWebAuthenticationSessionErrorCode::CanceledLogin.0 {
        "cancelled".to_string()
    } else {
        error.localizedDescription().to_string()
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// User dismissal must map to the exact `"cancelled"` string the frontend
    /// pattern-matches on (`oauth_buttons.rs`), not to a localized message.
    #[test]
    fn canceled_login_maps_to_cancelled_marker() {
        // CanceledLogin == 1 per ASWebAuthenticationSessionErrorCode.
        assert_eq!(ASWebAuthenticationSessionErrorCode::CanceledLogin.0, 1);
    }

    /// The wrapper exists precisely to cross the completion-handler thread
    /// boundary: if a future edit removes the `unsafe impl`, this fails to
    /// compile instead of failing at runtime with a poisoned slot.
    #[test]
    fn sendable_bundle_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SendableBundle>();
    }

    /// The slot hands its payload out exactly once and keeps working after
    /// a poisoned critical section: `take_slot` is the only reader, so a
    /// panic under the lock leaves no invariant to uphold.
    #[test]
    fn slot_take_is_once_and_poison_resistant() {
        let slot: SessionSlot = Arc::new(Mutex::new(None));

        // Empty slot stays empty on repeated takes.
        assert!(take_slot(&slot).is_none());
        assert!(take_slot(&slot).is_none());

        // Poison the mutex by panicking while holding the lock.
        let poisoned: SessionSlot = Arc::new(Mutex::new(None));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = poisoned.lock().unwrap();
            panic!("intentional poisoning");
        }));

        // take_slot still recovers the (empty) payload instead of
        // propagating the poison error.
        assert!(take_slot(&poisoned).is_none());
    }
}
