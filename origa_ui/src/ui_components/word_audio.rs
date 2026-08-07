use std::cell::RefCell;
use std::rc::Rc;

use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use origa::dictionary::pitch_audio::get_audio_for_reading;
use tracing::warn;
use wasm_bindgen_futures::JsFuture;

use super::{
    extract_japanese_text, get_reading_from_text, speak_tts_text, speak_tts_text_with_callback,
    stop_speech,
};
use crate::repository::cdn_provider::prefetch_blob_url;

struct ActiveAudio {
    element: web_sys::HtmlAudioElement,
    on_stop: Option<Box<dyn Fn()>>,
    _closures: Vec<Closure<dyn FnMut()>>,
}

thread_local! {
    static CURRENT_AUDIO: RefCell<Option<ActiveAudio>> = const { RefCell::new(None) };
}

pub fn stop_current_audio() {
    let prev = CURRENT_AUDIO.with(|cell| cell.borrow_mut().take());
    if let Some(active) = prev {
        if let Some(on_stop) = active.on_stop {
            on_stop();
        }
        active.element.set_onended(None);
        active.element.set_onerror(None);
        let _ = active.element.pause();
    }
    let _ = stop_speech();
}

pub fn register_audio(
    element: web_sys::HtmlAudioElement,
    on_stop: Option<Box<dyn Fn()>>,
    closures: Vec<Closure<dyn FnMut()>>,
) {
    CURRENT_AUDIO.with(|cell| {
        *cell.borrow_mut() = Some(ActiveAudio {
            element,
            on_stop,
            _closures: closures,
        });
    });
}

fn kata_to_hira(text: &str) -> String {
    text.chars()
        .map(|c| {
            // Standard katakana range (Katakana small aio -> Ka). Does not cover
            // extended katakana (ヷヸヹヺ) which never appear in dictionary readings.
            if ('\u{30A1}'..='\u{30F6}').contains(&c) {
                char::from_u32(c as u32 - 0x60).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

/// Resolve the CDN dictionary path for a word's pitch audio, if available.
fn lookup_audio_path(word: &str) -> Option<String> {
    let reading = get_reading_from_text(word);
    let reading_hira = kata_to_hira(&reading);
    let entry = get_audio_for_reading(word, &reading_hira)?;
    Some(format!("/{}", entry.cdn_path()))
}

/// Prefetch the blob URL for `word` and play it. `on_end` is invoked when the
/// audio finishes naturally (used by callback variants). If the lookup yields
/// no dictionary path, the function falls back to TTS synchronously.
fn schedule_word_audio_play<F>(word: &str, rate: f32, on_end: Option<F>)
where
    F: FnMut() + 'static,
{
    let Some(path) = lookup_audio_path(word) else {
        stop_current_audio();
        let reading = extract_japanese_text(&get_reading_from_text(word));
        match on_end {
            Some(cb) => {
                let _ = speak_tts_text_with_callback(&reading, rate, cb);
            },
            None => {
                let _ = speak_tts_text(&reading, rate);
            },
        }
        return;
    };

    let word_owned = word.to_string();
    let on_end_rc: Rc<RefCell<Option<F>>> = Rc::new(RefCell::new(on_end));
    // Stop synchronously before the async prefetch so a previously-playing word
    // does not overlap the new one during the network round-trip.
    stop_current_audio();
    spawn_local(async move {
        let blob_url = match prefetch_blob_url(&path).await {
            Ok(u) => u,
            Err(e) => {
                warn!(word = %word_owned, error = ?e, "CDN audio prefetch failed, falling back to TTS");
                fallback_to_tts(&word_owned, rate, &on_end_rc);
                return;
            },
        };

        let Some(audio) = web_sys::HtmlAudioElement::new().ok() else {
            fallback_to_tts(&word_owned, rate, &on_end_rc);
            return;
        };
        audio.set_src(&blob_url);
        let _ = audio.set_attribute("preload", "auto");

        let mut closures: Vec<Closure<dyn FnMut()>> = Vec::new();

        // on_ended and on_error race for the single-use on_end callback: whoever
        // fires first drains `on_end_rc` and either invokes the callback
        // directly (on_ended) or feeds it into the TTS fallback chain
        // (on_error) so downstream UI state is always released exactly once.
        let cb_ended = Rc::clone(&on_end_rc);
        let on_ended = Closure::<dyn FnMut()>::new(move || {
            if let Some(mut cb) = cb_ended.borrow_mut().take() {
                cb();
            }
        });
        audio.set_onended(Some(on_ended.as_ref().unchecked_ref()));
        closures.push(on_ended);

        let cb_error = Rc::clone(&on_end_rc);
        let word_for_error = word_owned.clone();
        let on_error = Closure::<dyn FnMut()>::new(move || {
            warn!(word = %word_for_error, "CDN audio failed during playback, falling back to TTS");
            fallback_to_tts(&word_for_error, rate, &cb_error);
        });
        audio.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        closures.push(on_error);

        register_audio(audio.clone(), None, closures);

        // Bug B fix: consume the play() Promise to avoid an uncaught rejection.
        // onerror does NOT fire for autoplay-policy NotAllowedError, so on
        // rejection we drain the callback and fall back to TTS to release any
        // downstream UI state.
        if let Ok(promise) = audio.play() {
            if JsFuture::from(promise).await.is_err() {
                warn!(word = %word_owned, "audio.play() rejected, falling back to TTS");
                fallback_to_tts(&word_owned, rate, &on_end_rc);
            }
        }
    });
}

/// Drain the optional on-end callback and trigger the TTS fallback chain.
fn fallback_to_tts<F>(word: &str, rate: f32, on_end: &Rc<RefCell<Option<F>>>)
where
    F: FnMut() + 'static,
{
    let reading = extract_japanese_text(&get_reading_from_text(word));
    if let Some(cb) = on_end.borrow_mut().take() {
        let _ = speak_tts_text_with_callback(&reading, rate, cb);
    } else {
        let _ = speak_tts_text(&reading, rate);
    }
}

pub fn speak_word(word: &str, rate: f32) {
    if word.is_empty() {
        return;
    }
    schedule_word_audio_play::<fn()>(word, rate, None);
}

pub fn speak_word_with_callback<F>(word: &str, rate: f32, on_end: F)
where
    F: FnMut() + 'static,
{
    if word.is_empty() {
        return;
    }
    schedule_word_audio_play(word, rate, Some(on_end));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kata_to_hira_converts_standard() {
        assert_eq!(kata_to_hira("ヤク"), "やく");
        assert_eq!(kata_to_hira("ア"), "あ");
        assert_eq!(kata_to_hira("ン"), "ん");
    }

    #[test]
    fn kata_to_hira_preserves_hiragana() {
        assert_eq!(kata_to_hira("やく"), "やく");
    }

    #[test]
    fn kata_to_hira_preserves_other() {
        assert_eq!(kata_to_hira("hello"), "hello");
        assert_eq!(kata_to_hira("123"), "123");
    }

    #[test]
    fn kata_to_hira_empty() {
        assert_eq!(kata_to_hira(""), "");
    }

    #[test]
    fn kata_to_hira_mixed() {
        assert_eq!(kata_to_hira("ヤクabc"), "やくabc");
    }

    #[test]
    fn kata_to_hira_long_vowel_mark() {
        assert_eq!(kata_to_hira("バー"), "ばー");
    }
}
