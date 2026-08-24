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
//! `AppHandle::run_on_main_thread` and awaits a oneshot channel. Apple
//! invokes the completion handler on the main queue as well; the handler
//! double-checks that and degrades to an error (never touching the
//! main-thread-bound slot) if that assumption is ever violated.

use std::cell::RefCell;
use std::rc::Rc;

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

/// Session + provider kept alive by the completion block (see the slot
/// comment in `start_session`).
type SessionBundle = (
    Retained<ASWebAuthenticationSession>,
    Retained<AuthAnchorProvider>,
);

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
    let bundle_slot: Rc<RefCell<Option<SessionBundle>>> = Rc::new(RefCell::new(None));

    let tx_for_completion = tx.clone();
    let bundle_for_completion = Rc::clone(&bundle_slot);
    let completion = RcBlock::new(move |callback_url: *mut NSURL, error: *mut NSError| {
        // Apple invokes the handler on the main queue. The check below keeps
        // the main-thread-only slot untouchable (and the process UB-free) if
        // that ever changes: we degrade to an error instead.
        let result = match MainThreadMarker::new() {
            None => Err("authentication completed off the main thread".to_string()),
            Some(_mtm) => autoreleasepool(|_pool| {
                // Break the session ↔ block retain cycle now that the system
                // is done with both.
                drop(bundle_for_completion.borrow_mut().take());
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
            }),
        };
        // Blocks are `Fn` (callable repeatedly by contract), so clone the
        // sender per invocation instead of moving it out of the closure.
        let _ = tx_for_completion.clone().send(result);
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

    *bundle_slot.borrow_mut() = Some((session.clone(), provider));

    // SAFETY: The session was created above; `start` has no additional safety
    // requirements beyond a valid receiver.
    if !unsafe { session.start() } {
        // No completion will fire — drop the bundle so the session and the
        // system's copy of the block deallocate instead of leaking.
        *bundle_slot.borrow_mut() = None;
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
}
