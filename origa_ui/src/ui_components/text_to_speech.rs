use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use crate::core::device_ai::{self, Feature, contracts::pick_japanese_voice};
use crate::core::tauri;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::closure::Closure;
use origa::domain::furiganize_segments;
use tracing::{info, warn};
use web_sys::js_sys::Function;
use web_sys::{SpeechSynthesisUtterance, SpeechSynthesisVoice, window};

thread_local! {
    static TTS_CALLBACK: RefCell<Option<Box<dyn FnMut()>>> = const { RefCell::new(None) };
    static TTS_LISTENER_REGISTERED: Cell<bool> = const { Cell::new(false) };
    static CACHED_VOICE_ID: RefCell<Option<String>> = const { RefCell::new(None) };
    static VOICE_RESOLVED: Cell<bool> = const { Cell::new(false) };
    // device-ai voice resolution result (None = not yet resolved / unavailable).
    static DEVICE_AI_VOICE_ID: RefCell<Option<String>> = const { RefCell::new(None) };
    static DEVICE_AI_VOICE_RESOLVED: Cell<bool> = const { Cell::new(false) };
}

fn ensure_tauri_listener_registered() {
    TTS_LISTENER_REGISTERED.with(|flag| {
        if flag.get() {
            return;
        }

        let Some(listen_fn) = tauri::event_listen_fn() else {
            return;
        };

        let closure = Closure::<dyn FnMut(JsValue)>::new(|_| {
            TTS_CALLBACK.with(|cell| {
                if let Some(mut cb) = cell.borrow_mut().take() {
                    cb();
                }
            });
        });

        let event_name = JsValue::from_str("tts://speech:finish");
        let _ = listen_fn.call2(&JsValue::UNDEFINED, &event_name, closure.as_ref());
        closure.forget();
        flag.set(true);
    });
}

async fn resolve_japanese_voice_id() -> Option<String> {
    let cached = CACHED_VOICE_ID.with(|cell| cell.borrow().clone());
    if let Some(id) = cached {
        return Some(id);
    }

    if VOICE_RESOLVED.with(|f| f.get()) {
        return None;
    }

    let invoke_fn = tauri::invoke_fn()?;

    let args = js_sys::Object::new();
    let result = invoke_fn
        .call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str("plugin:tts|get_voices"),
            &args,
        )
        .ok()?;

    let voices_js = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .ok()?;

    let voices_arr = js_sys::Array::from(&voices_js);

    let mut best_enhanced_kyoko: Option<String> = None;
    let mut best_compact_kyoko: Option<String> = None;
    let mut best_ja_jp: Option<String> = None;

    for voice_val in voices_arr.iter() {
        let id = js_sys::Reflect::get(&voice_val, &JsValue::from_str("id"))
            .ok()
            .and_then(|v| v.as_string());

        let name = js_sys::Reflect::get(&voice_val, &JsValue::from_str("name"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        let language = js_sys::Reflect::get(&voice_val, &JsValue::from_str("language"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        let Some(id) = id else { continue };

        if !language.starts_with("ja") {
            continue;
        }

        let name_lower = name.to_lowercase();
        let is_kyoko = name_lower.contains("kyoko");
        let is_enhanced = name_lower.contains("enhanced");

        if is_kyoko && is_enhanced && best_enhanced_kyoko.is_none() {
            best_enhanced_kyoko = Some(id.clone());
        }

        if is_kyoko && !is_enhanced && best_compact_kyoko.is_none() {
            best_compact_kyoko = Some(id.clone());
        }

        if best_ja_jp.is_none() {
            best_ja_jp = Some(id);
        }
    }

    let resolved = best_enhanced_kyoko.or(best_compact_kyoko).or(best_ja_jp);

    if let Some(ref id) = resolved {
        CACHED_VOICE_ID.with(|cell| *cell.borrow_mut() = Some(id.clone()));
    }

    VOICE_RESOLVED.with(|f| f.set(true));

    resolved
}

async fn invoke_tauri_speak(text: &str, rate: f32) -> Result<(), String> {
    let invoke_fn = tauri::invoke_fn().ok_or("Tauri invoke not available")?;

    let payload = js_sys::Object::new();
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("text"),
        &JsValue::from_str(text),
    )
    .map_err(|e| format!("Failed to set text: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("language"),
        &JsValue::from_str("ja-JP"),
    )
    .map_err(|e| format!("Failed to set language: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("rate"),
        &JsValue::from_f64(rate as f64),
    )
    .map_err(|e| format!("Failed to set rate: {:?}", e))?;
    let voice_id = resolve_japanese_voice_id().await;

    if let Some(ref vid) = voice_id {
        js_sys::Reflect::set(
            &payload,
            &JsValue::from_str("voiceId"),
            &JsValue::from_str(vid),
        )
        .map_err(|e| format!("Failed to set voiceId: {:?}", e))?;
    }

    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("pitch"),
        &JsValue::from_f64(1.2),
    )
    .map_err(|e| format!("Failed to set pitch: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("volume"),
        &JsValue::from_f64(1.0),
    )
    .map_err(|e| format!("Failed to set volume: {:?}", e))?;
    js_sys::Reflect::set(
        &payload,
        &JsValue::from_str("queueMode"),
        &JsValue::from_str("flush"),
    )
    .map_err(|e| format!("Failed to set queueMode: {:?}", e))?;

    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("payload"), &payload)
        .map_err(|e| format!("Failed to set payload: {:?}", e))?;

    let result = invoke_fn
        .call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str("plugin:tts|speak"),
            &args,
        )
        .map_err(|e| format!("invoke plugin:tts|speak failed: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .map(|_| ())
        .map_err(|e| format!("plugin:tts|speak error: {:?}", e))
}

async fn invoke_tauri_stop() -> Result<(), String> {
    let invoke_fn = tauri::invoke_fn().ok_or("Tauri invoke not available")?;

    let result = invoke_fn
        .call1(&JsValue::UNDEFINED, &JsValue::from_str("plugin:tts|stop"))
        .map_err(|e| format!("invoke plugin:tts|stop failed: {:?}", e))?;

    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result))
        .await
        .map(|_| ())
        .map_err(|e| format!("plugin:tts|stop error: {:?}", e))
}

/// Resolves a Japanese voice id from the native device-ai voice list.
/// Cached after the first successful or failed resolution.
async fn resolve_device_ai_voice_id() -> Option<String> {
    if let Some(id) = DEVICE_AI_VOICE_ID.with(|c| c.borrow().clone()) {
        return Some(id);
    }
    if DEVICE_AI_VOICE_RESOLVED.with(|f| f.get()) {
        return None;
    }

    let voices = device_ai::get_voices().await.ok()?;
    let picked = pick_japanese_voice(&voices).map(|v| v.id.clone());

    if let Some(ref id) = picked {
        DEVICE_AI_VOICE_ID.with(|c| *c.borrow_mut() = Some(id.clone()));
    }
    DEVICE_AI_VOICE_RESOLVED.with(|f| f.set(true));
    picked
}

/// Synthesize and play via native device-ai TTS.
async fn device_ai_speak(text: &str, rate: f32) -> Result<(), String> {
    let voice_id = resolve_device_ai_voice_id().await;
    device_ai::synthesize(text, voice_id.as_deref(), rate).await
}

pub fn is_speech_supported() -> bool {
    if tauri::is_tauri() {
        return true;
    }
    window().and_then(|w| w.speech_synthesis().ok()).is_some()
}

pub fn speak_tts_text(text: &str, rate: f32) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    if tauri::is_tauri() {
        let text_owned = text.to_string();
        spawn_local(async move {
            // device-ai native TTS is primary (macOS/iOS/Android); the legacy
            // plugin:tts is the fallback (Windows/Linux, or where device-ai is
            // unavailable). Routing is runtime-resolved via capabilities.
            let result = if device_ai::available(Feature::SpeechSynthesis).await {
                info!("TTS: using native device-ai synthesis");
                device_ai_speak(&text_owned, rate).await
            } else {
                info!("TTS: native device-ai unavailable, using legacy plugin:tts");
                invoke_tauri_speak(&text_owned, rate).await
            };
            if let Err(e) = result {
                warn!("TTS speak error: {}", e);
            }
        });
        return Ok(());
    }

    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;

    synthesis.cancel();

    let utterance = SpeechSynthesisUtterance::new()
        .map_err(|e| format!("Failed to create utterance: {:?}", e))?;
    utterance.set_text(text);
    utterance.set_rate(rate);
    utterance.set_lang("ja-JP");

    if let Some(voice) = get_japanese_voice(&synthesis) {
        utterance.set_voice(Some(&voice));
    }

    synthesis.speak(&utterance);
    Ok(())
}

pub fn speak_tts_text_with_callback<F>(text: &str, rate: f32, mut on_end: F) -> Result<(), String>
where
    F: FnMut() + 'static,
{
    if text.is_empty() {
        return Ok(());
    }

    if tauri::is_tauri() {
        let text_owned = text.to_string();
        spawn_local(async move {
            // device-ai synthesize is blocking — it resolves when playback
            // finishes, so the callback fires right after. The legacy
            // plugin:tts path instead emits `tts://speech:finish`, consumed by
            // the registered listener; device-ai never emits that event.
            if device_ai::available(Feature::SpeechSynthesis).await {
                info!("TTS: using native device-ai synthesis");
                if let Err(e) = device_ai_speak(&text_owned, rate).await {
                    warn!("TTS speak error: {}", e);
                }
                on_end();
            } else {
                info!("TTS: native device-ai unavailable, using legacy plugin:tts");
                TTS_CALLBACK.with(|cell| {
                    *cell.borrow_mut() = Some(Box::new(on_end));
                });
                ensure_tauri_listener_registered();

                if let Err(e) = invoke_tauri_speak(&text_owned, rate).await {
                    warn!("TTS speak error: {}", e);
                }
            }
        });
        return Ok(());
    }

    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;

    synthesis.cancel();

    let utterance = SpeechSynthesisUtterance::new()
        .map_err(|e| format!("Failed to create utterance: {:?}", e))?;
    utterance.set_text(text);
    utterance.set_rate(rate);
    utterance.set_lang("ja-JP");

    if let Some(voice) = get_japanese_voice(&synthesis) {
        utterance.set_voice(Some(&voice));
    }

    let closure = Closure::<dyn FnMut()>::new(on_end);
    if let Some(func) = closure.as_ref().dyn_ref::<Function>() {
        utterance.set_onend(Some(func));
    }
    closure.forget();

    synthesis.speak(&utterance);
    Ok(())
}

pub fn get_reading_from_text(text: &str) -> String {
    get_reading_from_text_with_known_kanji(text, &HashSet::new())
}

pub fn get_reading_from_text_with_known_kanji(text: &str, known_kanji: &HashSet<char>) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    furiganize_segments(text, known_kanji)
        .map(|segments| {
            segments
                .iter()
                .map(|seg| {
                    seg.reading()
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| seg.text().to_string())
                })
                .collect::<String>()
        })
        .unwrap_or_else(|_| text.to_string())
}

fn get_japanese_voice(synthesis: &web_sys::SpeechSynthesis) -> Option<SpeechSynthesisVoice> {
    let voices = synthesis.get_voices();

    let japanese_voices: Vec<SpeechSynthesisVoice> = voices
        .iter()
        .filter_map(|v| {
            let voice: SpeechSynthesisVoice = v.dyn_into().ok()?;
            if voice.lang().starts_with("ja") {
                Some(voice)
            } else {
                None
            }
        })
        .collect();

    japanese_voices
        .iter()
        .find(|v| v.name().to_lowercase().contains("kyoko"))
        .cloned()
        .or_else(|| {
            japanese_voices
                .iter()
                .find(|v| v.name().to_lowercase().contains("female"))
                .cloned()
        })
        .or(japanese_voices.into_iter().next())
}

/// Extract only Japanese characters from a mixed-language text.
///
/// Grammar card titles (and similar UI strings) embed Japanese alongside
/// native-language explanations, e.g. `"〜ば (условие)"`. Sending the full
/// string to TTS causes the engine to attempt reading the non-Japanese
/// portion with a Japanese voice, producing garbled output. This function
/// keeps hiragana, katakana, kanji (CJK Unified Ideographs), CJK punctuation,
/// the long vowel mark (ー), and whitespace between Japanese segments, so TTS
/// only receives the pronounceable portion.
pub fn extract_japanese_text(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            // Hiragana: 3040-309F
            ('\u{3040}'..='\u{309F}').contains(&c)
            // Katakana + long vowel mark + half-width katakana: 30A0-30FF
            || ('\u{30A0}'..='\u{30FF}').contains(&c)
            // CJK Unified Ideographs (common kanji): 4E00-9FFF
            || ('\u{4E00}'..='\u{9FFF}').contains(&c)
            // CJK Unified Ideographs Extension A (rare kanji): 3400-4DBF
            || ('\u{3400}'..='\u{4DBF}').contains(&c)
            // CJK punctuation (ideographic comma/period/fullwidth): 3000-303F
            || ('\u{3000}'..='\u{303F}').contains(&c)
            // Half-width katakana: FF65-FF9F
            || ('\u{FF65}'..='\u{FF9F}').contains(&c)
            // Whitespace (to keep Japanese segments separated naturally)
            || c.is_whitespace()
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn stop_speech() -> Result<(), String> {
    if tauri::is_tauri() {
        spawn_local(async {
            // device-ai TTS (both Android QUEUE_FLUSH and iOS
            // stopSpeaking) flushes the current utterance natively on the
            // next speak() call. The legacy plugin:tts stop is still called
            // for the Windows/Linux fallback path.
            if let Err(e) = invoke_tauri_stop().await {
                warn!("TTS stop error: {}", e);
            }
        });
        return Ok(());
    }

    let window = window().ok_or("Window not available")?;
    let synthesis = window
        .speech_synthesis()
        .map_err(|e| format!("Speech synthesis not supported: {:?}", e))?;
    synthesis.cancel();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_japanese_from_mixed_text() {
        assert_eq!(extract_japanese_text("〜ば (условие)"), "〜ば");
    }

    #[test]
    fn extract_japanese_keeps_katakana_and_kanji() {
        assert_eq!(extract_japanese_text("食べる taberu"), "食べる");
    }

    #[test]
    fn extract_japanese_keeps_long_vowel_mark() {
        assert_eq!(extract_japanese_text("コーヒー coffee"), "コーヒー");
    }

    #[test]
    fn extract_japanese_pure_japanese_unchanged() {
        assert_eq!(extract_japanese_text("日本語"), "日本語");
    }

    #[test]
    fn extract_japanese_no_japanese_returns_empty() {
        assert_eq!(extract_japanese_text("hello world"), "");
    }

    #[test]
    fn extract_japanese_empty_string() {
        assert_eq!(extract_japanese_text(""), "");
    }

    #[test]
    fn extract_japanese_keeps_cjk_punctuation() {
        assert_eq!(extract_japanese_text("大きい。big"), "大きい。");
    }

    #[test]
    fn extract_japanese_multiple_segments() {
        assert_eq!(
            extract_japanese_text("〜ば条件、if condition"),
            "〜ば条件、"
        );
    }

    #[test]
    fn extract_japanese_hiragana_only() {
        assert_eq!(extract_japanese_text("ありがとう thank you"), "ありがとう");
    }
}
