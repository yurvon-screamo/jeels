//! Autoplay policy unlock on first user gesture.
//!
//! Browser and Tauri WebView autoplay policies block `HTMLAudioElement.play()`
//! and `speechSynthesis.speak()` until the page has user activation. Lesson
//! cards auto-play audio from a reactive `Effect` (no user gesture), so audio
//! on the question side is frequently silent until the user clicks something.
//! Manual playback (AudioButtons) and the answer side work because both are
//! driven by a real gesture. This module unlocks playback once, on the first
//! `pointerdown` or `keydown`, using two independent mechanisms because the
//! two playback paths are governed by different policies:
//!
//! - **HTMLMediaElement** (CDN phrase/pitch audio): governed by Media
//!   Engagement Index + user activation. `AudioContext.resume()` does NOT
//!   unlock it — Origa plays through raw `HTMLAudioElement.play()`, not a Web
//!   Audio graph. The unlock is a silent `play()`+`pause()` of a real WAV
//!   inside the gesture.
//! - **speechSynthesis** (web TTS): unlocked by a silent utterance (`volume
//!   0.0`) inside the gesture, which also primes the asynchronously-populated
//!   Chromium voice list.
//!
//! Limitation: a deep-link/reload that lands directly on a lesson renders the
//! first card before any gesture, so that card's audio is skipped — a
//! fundamental browser restriction. Lessons advance manually, so sticky
//! activation holds across subsequent cards.

use std::cell::Cell;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::use_event_listener;
use tracing::warn;
use wasm_bindgen_futures::JsFuture;
use web_sys::SpeechSynthesisUtterance;

/// Minimal valid silent WAV: 44-byte RIFF/WAVE header + a single zero PCM
/// sample. A real decodable fragment is required — a 0-byte or empty src can
/// reject `play()` and fail to register user activation for the session.
const SILENT_WAV_SRC: &str =
    "data:audio/wav;base64,UklGRiYAAABXQVZFZm10IBAAAAABAAEARKwAAIhYAQACABAAZGF0YQIAAAAAAA==";

thread_local! {
    static AUDIO_UNLOCKED: Cell<bool> = const { Cell::new(false) };
}

/// Claim the one-shot unlock against a flag. Returns `true` the first time the
/// flag is `false` (and flips it), `false` thereafter. Pure logic over a
/// borrowed flag so the once-semantics is unit-testable without a browser.
pub(crate) fn claim_audio_unlock(flag: &Cell<bool>) -> bool {
    if flag.get() {
        false
    } else {
        flag.set(true);
        true
    }
}

/// Register global `pointerdown` and `keydown` listeners (on `window`) that run
/// the one-shot unlock on the first user gesture. Lesson interaction is often
/// keyboard-first (Space/Enter to show the answer, key-based rating), so both
/// events are gated by a single flag: whichever fires first performs the
/// unlock and the other becomes a no-op. `use_event_listener` auto-cleans up
/// on the reactive owner; `App()` owns the listeners for the whole app
/// lifetime.
pub(crate) fn install_audio_unlock() {
    let _ = use_event_listener(window(), leptos::ev::pointerdown, move |_| {
        run_unlock_once();
    });
    let _ = use_event_listener(window(), leptos::ev::keydown, move |_| {
        run_unlock_once();
    });
}

fn run_unlock_once() {
    let claimed = AUDIO_UNLOCKED.with(claim_audio_unlock);
    if claimed {
        unlock_html_media_element();
        warm_up_speech_synthesis();
    }
}

/// Silent `play()`+`pause()` of a real WAV inside the gesture to register
/// media-engagement/user-activation for `HTMLMediaElement` on the session.
fn unlock_html_media_element() {
    let Ok(audio) = web_sys::HtmlAudioElement::new_with_src(SILENT_WAV_SRC) else {
        warn!("audio unlock: HtmlAudioElement creation failed");
        return;
    };

    match audio.play() {
        Ok(promise) => {
            // play() runs synchronously inside the user gesture — that is what
            // registers activation. Pause once the promise resolves to keep
            // the unlock inaudible.
            spawn_local(async move {
                if JsFuture::from(promise).await.is_err() {
                    warn!("audio unlock: silent play() rejected");
                    return;
                }
                let _ = audio.pause();
                audio.set_src("");
            });
        },
        Err(_) => warn!("audio unlock: play() threw synchronously"),
    }
}

/// Speak a silent utterance inside the gesture to unlock `speechSynthesis` and
/// prime the asynchronously-populated Chromium voice list. Web-build only — in
/// Tauri, TTS goes through the native `plugin:tts` (not subject to the browser
/// autoplay policy), so the web `speechSynthesis` path is never used there.
fn warm_up_speech_synthesis() {
    if crate::core::tauri::is_tauri() {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(synthesis) = window.speech_synthesis() else {
        return;
    };
    let Ok(utterance) = SpeechSynthesisUtterance::new() else {
        warn!("audio unlock: SpeechSynthesisUtterance creation failed");
        return;
    };
    // Non-empty text is required for some engines to register the utterance;
    // volume 0 keeps it inaudible.
    utterance.set_text(" ");
    utterance.set_volume(0.0);
    utterance.set_lang("ja-JP");
    synthesis.speak(&utterance);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_claim_returns_true_and_sets_flag() {
        let flag = Cell::new(false);

        let claimed = claim_audio_unlock(&flag);

        assert!(claimed);
        assert!(flag.get());
    }

    #[test]
    fn subsequent_claims_return_false() {
        let flag = Cell::new(false);

        let first = claim_audio_unlock(&flag);
        let second = claim_audio_unlock(&flag);
        let third = claim_audio_unlock(&flag);

        assert!(first);
        assert!(!second);
        assert!(!third);
        assert!(flag.get());
    }
}
