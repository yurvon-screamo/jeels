//! Component-level tests for `LessonCard` using `wasm_bindgen_test`.
//!
//! These tests mount the real Leptos component into a headless browser DOM
//! and assert on the rendered HTML — catching bugs that `cargo test` (no
//! rendering) and BDD (test_id-only assertions) miss, such as reversed
//! cards showing Japanese on both the question and answer screens.
//!
//! Run locally:
//! ```bash
//! wasm-pack test --headless --chrome origa_ui -- --features csr -- lesson_card_wasm
//! ```

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use origa::domain::{
    Card as DomainCard, GrammarInfo, NativeLanguage, PartOfSpeech, Question, VocabularyCard,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use super::lesson_card::LessonCard;

// ─── Helpers ───────────────────────────────────────────────────────────

/// Empty known-kanji set (no kanji marked as known).
fn empty_known_kanji() -> Signal<HashSet<char>> {
    Signal::derive(|| HashSet::new())
}

/// Mount a `LessonCard` into an isolated `<div>` appended to `<body>`.
/// Returns the wrapper element for DOM queries.
fn mount_lesson_card(
    card: DomainCard,
    is_reversed: bool,
    show_answer: bool,
    grammar_info: Option<GrammarInfo>,
) -> web_sys::Element {
    // Install panic hook so panics inside Effects are visible in console.
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let wrapper = document.create_element("div").unwrap();
    let _ = document.body().unwrap().append_child(&wrapper);

    let show_answer_signal = Signal::from(show_answer);
    let on_show_answer = Callback::new(|_: ()| {});

    let _dispose = leptos::mount::mount_to(
        wrapper.clone().unchecked_into(),
        move || {
            // Provide i18n context inside the reactive Owner scope so
            // components calling use_i18n() and Effect::new don't panic.
            leptos_i18n::provide_i18n_context::<crate::i18n::Locale>();

            view! {
                <LessonCard
                    card=card.clone()
                    show_answer=show_answer_signal
                    is_reversed=is_reversed
                    on_show_answer=on_show_answer
                    grammar_info=grammar_info.clone()
                    native_language=NativeLanguage::Russian
                    known_kanji=empty_known_kanji()
                    audio_path=None
                />
            }
        },
    );

    wrapper
}

/// Create a normal (non-reversed) vocabulary card.
/// `word` is Japanese; `translation` is placed in `reverse_side` so
/// `answer()` returns it without needing a loaded translation dictionary.
fn normal_vocab_card(word: &str, translation: &str) -> DomainCard {
    DomainCard::Vocabulary(VocabularyCard::new_with_pos(
        Question::new(word.to_string()).unwrap(),
        None,
        Some(Question::new(translation.to_string()).unwrap()),
    ))
}

/// Create a na-adjective card (non-reversed).
fn normal_na_adj_card(word: &str, translation: &str) -> DomainCard {
    DomainCard::Vocabulary(VocabularyCard::new_with_pos(
        Question::new(word.to_string()).unwrap(),
        Some(PartOfSpeech::NaAdjective),
        Some(Question::new(translation.to_string()).unwrap()),
    ))
}

/// Create a reversed vocabulary card.
/// `translation` becomes the question (word); original Japanese goes
/// into `reverse_side` so `answer()` returns it.
fn reversed_vocab_card(word_japanese: &str, translation: &str) -> DomainCard {
    DomainCard::Vocabulary(VocabularyCard::new_with_pos(
        Question::new(translation.to_string()).unwrap(),
        None,
        Some(Question::new(word_japanese.to_string()).unwrap()),
    ))
}

/// Create a reversed na-adjective card.
fn reversed_na_adj_card(word_japanese: &str, translation: &str) -> DomainCard {
    DomainCard::Vocabulary(VocabularyCard::new_with_pos(
        Question::new(translation.to_string()).unwrap(),
        Some(PartOfSpeech::NaAdjective),
        Some(Question::new(word_japanese.to_string()).unwrap()),
    ))
}

// ─── Matrix: Normal vocab card ─────────────────────────────────────────

#[wasm_bindgen_test]
async fn minimal_mount_works() {
    console_error_panic_hook::set_once();
    let document = web_sys::window().unwrap().document().unwrap();
    let wrapper = document.create_element("div").unwrap();
    let _ = document.body().unwrap().append_child(&wrapper);

    let _dispose = leptos::mount::mount_to(
        wrapper.clone().unchecked_into(),
        || view! { <div class="test-mount">"Hello WASM"</div> },
    );
    tick().await;

    let html = wrapper.inner_html();
    web_sys::console::log_1(&format!("MINIMAL HTML: {html}").into());
    assert!(html.contains("Hello WASM"), "minimal mount must work; got: {html}");
}

#[wasm_bindgen_test]
async fn normal_vocab_question_shows_japanese() {
    let card = normal_vocab_card("猫", "кошка");
    let wrapper = mount_lesson_card(card, false, false, None);
    tick().await;

    // Debug: log what was actually rendered
    let html = wrapper.inner_html();
    let body_html = web_sys::window().unwrap().document().unwrap().body().unwrap().inner_html();
    web_sys::console::log_1(&format!("WRAPPER HTML: {html}").into());
    web_sys::console::log_1(&format!("BODY HTML (first 500): {}", &body_html[..500.min(body_html.len())]).into());

    let question = wrapper
        .query_selector(".lesson-question")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        question.contains("猫"),
        "normal vocab question must show Japanese word; got: {question}"
    );
}

#[wasm_bindgen_test]
async fn normal_vocab_answer_heading_shows_japanese() {
    let card = normal_vocab_card("猫", "кошка");
    let wrapper = mount_lesson_card(card, false, true, None);
    tick().await;

    let heading = wrapper
        .query_selector("h3")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        heading.contains("猫"),
        "normal vocab answer heading must show Japanese; got: {heading}"
    );
}

#[wasm_bindgen_test]
async fn normal_vocab_answer_body_shows_translation() {
    let card = normal_vocab_card("猫", "кошка");
    let wrapper = mount_lesson_card(card, false, true, None);
    tick().await;

    let body = wrapper.inner_html();
    assert!(
        body.contains("кошка"),
        "normal vocab answer body must contain translation; got HTML: {body}"
    );
}

// ─── Matrix: Normal na-adjective card ──────────────────────────────────

#[wasm_bindgen_test]
async fn normal_na_adj_question_shows_japanese_with_na_suffix() {
    let card = normal_na_adj_card("静か", "тихий");
    let wrapper = mount_lesson_card(card, false, false, None);
    tick().await;

    let question = wrapper
        .query_selector(".lesson-question")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        question.contains("静かな"),
        "normal na-adj question must show Japanese + な suffix; got: {question}"
    );
}

#[wasm_bindgen_test]
async fn normal_na_adj_answer_heading_shows_japanese_with_na() {
    let card = normal_na_adj_card("静か", "тихий");
    let wrapper = mount_lesson_card(card, false, true, None);
    tick().await;

    let heading = wrapper
        .query_selector("h3")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        heading.contains("静かな"),
        "normal na-adj answer heading must show Japanese + な; got: {heading}"
    );
}

#[wasm_bindgen_test]
async fn normal_na_adj_answer_body_shows_translation_without_na() {
    let card = normal_na_adj_card("静か", "тихий");
    let wrapper = mount_lesson_card(card, false, true, None);
    tick().await;

    let body = wrapper.inner_html();
    assert!(
        !body.contains("тихийな"),
        "na-adj answer body must NOT have な suffix on translation; got HTML: {body}"
    );
}

// ─── Matrix: Reversed vocab card ───────────────────────────────────────

#[wasm_bindgen_test]
async fn reversed_vocab_question_shows_translation() {
    let card = reversed_vocab_card("猫", "кошка");
    let wrapper = mount_lesson_card(card, true, false, None);
    tick().await;

    let question = wrapper
        .query_selector(".lesson-question")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        question.contains("кошка"),
        "reversed vocab question must show translation; got: {question}"
    );
    assert!(
        !question.contains("猫"),
        "reversed vocab question must NOT show Japanese; got: {question}"
    );
}

#[wasm_bindgen_test]
async fn reversed_vocab_answer_heading_shows_japanese() {
    let card = reversed_vocab_card("猫", "кошка");
    let wrapper = mount_lesson_card(card, true, true, None);
    tick().await;

    let heading = wrapper
        .query_selector("h3")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        heading.contains("猫"),
        "reversed vocab answer heading must show Japanese; got: {heading}"
    );
}

#[wasm_bindgen_test]
async fn reversed_vocab_answer_body_shows_translation() {
    let card = reversed_vocab_card("猫", "кошка");
    let wrapper = mount_lesson_card(card, true, true, None);
    tick().await;

    let body = wrapper.inner_html();
    assert!(
        body.contains("кошка"),
        "reversed vocab answer body must contain translation; got HTML: {body}"
    );
}

// ─── Matrix: Reversed na-adjective card ────────────────────────────────

#[wasm_bindgen_test]
async fn reversed_na_adj_question_shows_translation_without_na() {
    let card = reversed_na_adj_card("静か", "тихий");
    let wrapper = mount_lesson_card(card, true, false, None);
    tick().await;

    let question = wrapper
        .query_selector(".lesson-question")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        question.contains("тихий"),
        "reversed na-adj question must show translation; got: {question}"
    );
    assert!(
        !question.contains("静かな"),
        "reversed na-adj question must NOT show Japanese + な; got: {question}"
    );
}

#[wasm_bindgen_test]
async fn reversed_na_adj_answer_heading_shows_japanese_without_na() {
    let card = reversed_na_adj_card("静か", "тихий");
    let wrapper = mount_lesson_card(card, true, true, None);
    tick().await;

    let heading = wrapper
        .query_selector("h3")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    // Reversed na-adj: heading is Japanese answer but WITHOUT な — な is only
    // appended to non-reversed question text (is_na_adj guard has !is_reversed).
    assert!(
        heading.contains("静か"),
        "reversed na-adj answer heading must show Japanese; got: {heading}"
    );
    assert!(
        !heading.contains("静かな"),
        "reversed na-adj answer heading must NOT have な suffix; got: {heading}"
    );
}

#[wasm_bindgen_test]
async fn reversed_na_adj_answer_body_shows_translation_without_na() {
    let card = reversed_na_adj_card("静か", "тихий");
    let wrapper = mount_lesson_card(card, true, true, None);
    tick().await;

    let body = wrapper.inner_html();
    assert!(
        !body.contains("тихийな"),
        "reversed na-adj answer body must NOT have な on translation; got HTML: {body}"
    );
}
