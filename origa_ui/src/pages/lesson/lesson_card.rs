use crate::ui_components::{
    Card, get_reading_from_text, is_speech_supported, register_audio, speak_tts_text, speak_word,
    stop_current_audio,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::closure::Closure;
use origa::domain::{Card as DomainCard, GrammarInfo, NativeLanguage};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use tracing;

use super::answer_display::extract_card_answer;
use super::card_type::CardType;
use super::kanji_card_details::RadicalDisplay;
use super::lesson_card_answer::LessonCardAnswer;
use super::lesson_card_header::{CardHeaderAudio, LessonCardTags};
use super::lesson_card_question::LessonCardQuestion;
use super::lesson_state::LessonContext;
use crate::repository::cdn_provider::prefetch_blob_url;

#[component]
pub fn LessonCard(
    card: DomainCard,
    show_answer: Signal<bool>,
    is_reversed: bool,
    on_show_answer: Callback<()>,
    grammar_info: Option<GrammarInfo>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    /// CDN path for the phrase audio (e.g. `phrases/audio/ABC.opus`).
    /// Prefetched into a `blob:` URL before playback — see
    /// `cdn_provider::resolve_audio_url` for the gzip-on-CDN root cause.
    #[prop(into)]
    audio_path: Option<String>,
) -> impl IntoView {
    let card_type = CardType::from(&card);
    let is_phrase = card_type == CardType::Phrase;
    let lang = native_language;
    // な-suffix is appended only to Japanese text, never to translations.
    // On a normal card, question_text IS the Japanese word → safe to append.
    // On a reversed card, question_text is the translation → must NOT append.
    let is_na_adj = !is_reversed
        && grammar_info.is_none()
        && super::na_adjective_helper::is_na_adjective_card(&card);

    let question_text = match card.question(&lang) {
        Ok(q) => q.text().to_string(),
        Err(e) => {
            tracing::warn!(
                card_type = ?card_type,
                content_key = %card.content_key(),
                error = %e,
                "Failed to get card question"
            );
            String::new()
        },
    };
    let question = StoredValue::new(question_text.clone());
    let display_question = StoredValue::new(if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&question_text)
    } else {
        question_text.clone()
    });

    let answer_data = extract_card_answer(&card, &lang, &card_type);
    let tts_text = answer_data
        .translations
        .as_ref()
        .map(|t| t.join(", "))
        .unwrap_or_else(|| answer_data.text.clone());

    // Question screen: normal → Japanese word (+な for na-adj);
    // reversed → translation (the user's prompt to recall the Japanese word).
    let card_question_text = StoredValue::new(if is_reversed {
        question_text.clone()
    } else if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&question_text)
    } else {
        question_text.clone()
    });
    // Answer heading: normal → Japanese word (+な for na-adj);
    // reversed → Japanese word from the answer side (tts_text = the
    // original Japanese stored in reverse_side, surfaced as CardAnswer::Text).
    let answer_heading = StoredValue::new(if is_reversed {
        tts_text.clone()
    } else if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&question_text)
    } else {
        question_text.clone()
    });
    let answer = StoredValue::new(tts_text);
    let answer_translations_stored = StoredValue::new(answer_data.translations);
    let answer_description_stored = StoredValue::new(answer_data.description);

    let radicals: Option<Vec<RadicalDisplay>> = match &card {
        DomainCard::Kanji(kanji) => match kanji.radicals_info() {
            Ok(r) => Some(
                r.iter()
                    .map(|info| RadicalDisplay {
                        symbol: info.radical(),
                        name: info.name(&lang).to_string(),
                        description: info.description(&lang).to_string(),
                    })
                    .collect(),
            ),
            Err(e) => {
                tracing::warn!("Failed to get radicals for kanji: {:?}", e);
                None
            },
        },
        _ => None,
    };
    let radicals_stored = StoredValue::new(radicals);

    let example_words: Option<Vec<(String, String)>> = match &card {
        DomainCard::Kanji(kanji) => {
            let examples: Vec<_> = kanji
                .example_words(&lang)
                .iter()
                .map(|e| (e.word().to_string(), e.meaning().to_string()))
                .collect();
            if examples.is_empty() {
                None
            } else {
                Some(examples)
            }
        },
        _ => None,
    };
    let examples_stored = StoredValue::new(example_words);

    let on_readings: Option<Vec<crate::ui_components::ReadingItem>> = match &card {
        DomainCard::Kanji(kanji) => {
            let readings = kanji
                .on_readings_with_freq()
                .into_iter()
                .map(|(r, f, rare)| crate::ui_components::ReadingItem {
                    reading: r,
                    freq: f,
                    is_rare: rare,
                })
                .collect::<Vec<_>>();
            if readings.is_empty() {
                None
            } else {
                Some(readings)
            }
        },
        _ => None,
    };
    let on_readings_stored = StoredValue::new(on_readings);

    let kun_readings: Option<Vec<crate::ui_components::ReadingItem>> = match &card {
        DomainCard::Kanji(kanji) => {
            let readings = kanji
                .kun_readings_with_freq()
                .into_iter()
                .map(|(r, f, rare)| crate::ui_components::ReadingItem {
                    reading: r,
                    freq: f,
                    is_rare: rare,
                })
                .collect::<Vec<_>>();
            if readings.is_empty() {
                None
            } else {
                Some(readings)
            }
        },
        _ => None,
    };
    let kun_readings_stored = StoredValue::new(kun_readings);

    let kanji_for_animation: Option<String> = match &card {
        DomainCard::Kanji(_) => {
            let text = match card.question(&lang) {
                Ok(q) => q.text().to_string(),
                Err(e) => {
                    tracing::warn!(
                        card_type = ?card_type,
                        content_key = %card.content_key(),
                        error = %e,
                        "Failed to get kanji for animation"
                    );
                    String::new()
                },
            };
            Some(text)
        },
        _ => None,
    };
    let kanji_stored = StoredValue::new(kanji_for_animation);

    let lesson_ctx = use_context::<LessonContext>();

    let is_expanded = RwSignal::new(card_type == CardType::Kanji);
    let content_ref = NodeRef::<leptos::html::Div>::new();
    let needs_collapse = RwSignal::new(false);

    let lesson_ctx_tts_normal = lesson_ctx.clone();
    Effect::new(move |_| {
        let is_muted = lesson_ctx_tts_normal
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if !show_answer.get()
            && !is_reversed
            && card_type != CardType::Kanji
            && card_type != CardType::Phrase
            && is_speech_supported()
            && !is_muted
        {
            speak_word(&question.get_value(), 1.0);
        }
    });

    let phrase_audio_path = audio_path.clone();
    let lesson_ctx_phrase = lesson_ctx.clone();

    Effect::new(move |_| {
        let is_muted = lesson_ctx_phrase
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if !show_answer.get() && is_phrase && !is_muted {
            if let Some(path) = phrase_audio_path.as_ref() {
                stop_current_audio();
                let path_owned = path.clone();
                let question_val = question.get_value();
                // Bug A fix: prefetch CDN bytes into a blob: URL before playback.
                spawn_local(async move {
                    play_phrase_in_lesson(&path_owned, &question_val).await;
                });
            }
        } else if is_phrase || show_answer.get() {
            stop_current_audio();
        }
    });

    let lesson_ctx_tts_reversed = lesson_ctx.clone();
    Effect::new(move |_| {
        let is_muted = lesson_ctx_tts_reversed
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if show_answer.get()
            && is_reversed
            && card_type != CardType::Kanji
            && is_speech_supported()
            && !is_muted
        {
            speak_word(&answer.get_value(), 1.0);
        }
    });

    Effect::new(move |_| {
        if show_answer.get()
            && card_type != CardType::Kanji
            && let Some(el) = content_ref.get()
        {
            // Only show "Expand" when content significantly overflows the
            // clamp. A small overflow (e.g. half a line) produces a button
            // that reveals almost nothing — confusing UX. The 1.3× threshold
            // means the collapsed view hides at least 30% of the content.
            let full = el.scroll_height() as f64;
            let visible = el.client_height() as f64;
            let is_significant_overflow = visible > 0.0 && full > visible * 1.3;
            needs_collapse.set(is_significant_overflow);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let on_toggle = Callback::new(move |()| {
        is_expanded.update(|v| *v = !*v);
    });

    let audio_path_stored = StoredValue::new(audio_path.clone());

    let audio_props = Signal::derive(move || {
        if is_reversed && show_answer.get() {
            Some(CardHeaderAudio {
                card_type,
                question_text: answer.get_value(),
                is_reversed: true,
                audio_path: audio_path_stored.get_value(),
            })
        } else if !is_reversed {
            Some(CardHeaderAudio {
                card_type,
                question_text: display_question.get_value(),
                is_reversed: false,
                audio_path: audio_path_stored.get_value(),
            })
        } else {
            None
        }
    });

    view! {
        <div class="flex flex-col">
            <LessonCardTags
                card_type=card_type
                grammar_info=grammar_info.clone()
                show_answer=show_answer
                card=card.clone()
                audio=audio_props
            />
            <Card class=Signal::derive(|| super::LESSON_CARD_CLASS.to_string()) shadow=true test_id="lesson-card-root">
                <div class="flex-1 flex flex-col justify-center">
                    <Show when=move || !show_answer.get()>
                        <LessonCardQuestion
                            question_text=card_question_text.get_value()
                            kanji=kanji_stored.get_value()
                            is_reversed=is_reversed
                            on_show_answer=on_show_answer
                            known_kanji=known_kanji
                            native_language=native_language
                        />
                    </Show>

                    <Show when=move || show_answer.get()>
                        <LessonCardAnswer
                            question_text=answer_heading.get_value()
                            answer_text=if is_reversed { question.get_value() } else { answer.get_value() }
                            answer_translations=answer_translations_stored.get_value()
                            answer_description=answer_description_stored.get_value()
                            is_expanded=is_expanded
                            needs_collapse=needs_collapse
                            content_ref=content_ref
                            on_toggle=on_toggle
                            is_kanji=card_type == CardType::Kanji
                            is_phrase
                            is_reversed=is_reversed
                            on_readings=on_readings_stored.get_value()
                            kun_readings=kun_readings_stored.get_value()
                            radicals=radicals_stored.get_value()
                            example_words=examples_stored.get_value()
                            grammar_info=grammar_info.clone()
                            known_kanji=known_kanji
                            native_language=native_language
                        />
                    </Show>
                </div>
            </Card>
        </div>
    }
}

/// Prefetch the phrase audio into a `blob:` URL and play it through a
/// fresh `HTMLAudioElement`, falling back to TTS on any failure. See
/// `cdn_provider::resolve_audio_url` for the Bug A/B root cause.
async fn play_phrase_in_lesson(path: &str, question_text: &str) {
    use wasm_bindgen_futures::JsFuture;

    // Shared single-shot TTS fallback used by the drain-pattern below.
    type TtsFallback = Rc<RefCell<Option<Box<dyn FnMut()>>>>;

    let blob_url = match prefetch_blob_url(path).await {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!(path = %path, error = ?e, "CDN phrase audio prefetch failed, falling back to TTS");
            let reading = get_reading_from_text(question_text);
            let _ = speak_tts_text(&reading, 1.0);
            return;
        },
    };

    let Ok(audio) = web_sys::HtmlAudioElement::new_with_src(&blob_url) else {
        let reading = get_reading_from_text(question_text);
        let _ = speak_tts_text(&reading, 1.0);
        return;
    };
    let _ = audio.set_attribute("preload", "auto");

    let question_owned = question_text.to_string();
    let path_owned = path.to_string();

    // Drain-pattern: Chromium emits BOTH an `error` event AND a play() Promise
    // rejection on decode failure, so calling TTS independently in onerror and
    // in the rejection branch would speak twice. The first branch to fire
    // drains the shared Option; the second observes `None` and is a no-op.
    let tts_fallback: TtsFallback = {
        let question_for_tts = question_owned.clone();
        Rc::new(RefCell::new(Some(Box::new(move || {
            let reading = get_reading_from_text(&question_for_tts);
            let _ = speak_tts_text(&reading, 1.0);
        }))))
    };

    let tts_for_error = Rc::clone(&tts_fallback);
    let on_error = Closure::<dyn FnMut()>::new(move || {
        tracing::warn!(path = %path_owned, "CDN phrase audio failed during playback, falling back to TTS");
        if let Some(mut cb) = tts_for_error.borrow_mut().take() {
            cb();
        }
    });
    audio.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    register_audio(audio.clone(), None, vec![on_error]);

    // Bug B fix: consume the play() Promise. onerror does NOT fire for
    // autoplay-policy NotAllowedError, so on rejection we still need to fire
    // TTS. We drain the shared fallback rather than calling TTS directly: on
    // decode errors onerror has already drained it, preventing the double-TTS.
    if let Ok(promise) = audio.play() {
        if JsFuture::from(promise).await.is_err() {
            tracing::warn!(path = %path, "audio.play() rejected, falling back to TTS");
            if let Some(mut cb) = tts_fallback.borrow_mut().take() {
                cb();
            }
        }
    }
}
