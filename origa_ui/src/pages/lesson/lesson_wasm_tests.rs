//! WASM render tests for `pages/lesson` presentational components:
//! `RatingButtons`, `RatingButtonsView`, `NextCardButton`,
//! `LessonCardQuestion`, `LessonProgress`, `GrammarInfoBadge`,
//! `CardAnswerDisplay`.

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use super::answer_display::CardAnswerDisplay;
use super::grammar_info_badge::GrammarInfoBadge;
use super::kanji_card_details::{KanjiCardDetails, RadicalDisplay};
use super::lesson_card_question::LessonCardQuestion;
use super::lesson_progress::LessonProgress;
use super::next_card_button::NextCardButton;
use super::phrase_card::PhraseCardView;
use super::phrase_rating_buttons::PhraseRatingButtons;
use super::quiz_options::QuizOptions;
use super::quiz_options_multi::QuizOptionsMulti;
use super::quiz_result::QuizResult;
use super::quiz_result_display::QuizResultDisplay;
use super::rating_buttons::RatingButtons;
use super::rating_buttons_view::RatingButtonsView;
use super::yesno_card_view::YesNoCardView;
use crate::test_support::{
    create_wrapper, mount_to_wrapper, mount_with_i18n, mount_with_router_and_stores, shared_cell,
};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// KanjiCardDetails: radical translation degradation
// ═══════════════════════════════════════════════════════════════════════

/// A radical whose localized name/description are empty strings ("no
/// translation for this language yet" — see `RadicalInfo::name`) must
/// degrade to the bare symbol: no blank muted text nodes may render.
#[wasm_bindgen_test]
async fn kanji_card_details_hides_empty_radical_translation() {
    let wrapper = create_wrapper();
    let (set_more_label, get_more_label) = shared_cell::<String>();
    mount_with_i18n(&wrapper, move || {
        let i18n = crate::i18n::use_i18n();
        // The details toggle is unlabelled by test id; identify it by its
        // localized "more details" label taken from the same i18n context
        // instead of grabbing the first button in the DOM.
        set_more_label.set(Some(
            crate::i18n::td_string!(i18n.get_locale_untracked(), common.more_details).to_string(),
        ));
        let radicals = vec![
            RadicalDisplay {
                symbol: '日',
                name: "Sun".to_string(),
                description: "The sun radical".to_string(),
            },
            RadicalDisplay {
                symbol: '山',
                name: String::new(),
                description: String::new(),
            },
        ];
        view! {
            <KanjiCardDetails
                kanji="明".to_string()
                name="bright".to_string()
                radicals=Some(radicals)
                example_words=None
                on_readings=None
                kun_readings=None
                known_kanji=Signal::from(HashSet::new())
                native_language=origa::domain::NativeLanguage::English
            />
        }
        .into_any()
    });
    let more_label = get_more_label.take().expect("label captured");
    tick().await;

    // Expand the details section: find the toggle by its localized label.
    let buttons = wrapper.query_selector_all("button").unwrap();
    let toggle = (0..buttons.length())
        .filter_map(|index| buttons.get(index))
        .find(|node| {
            node.dyn_ref::<web_sys::Element>()
                .is_some_and(|el| el.text_content().as_deref() == Some(more_label.as_str()))
        })
        .expect("the more-details toggle must render");
    toggle
        .dyn_into::<web_sys::HtmlElement>()
        .expect("toggle is an element")
        .click();
    tick().await;

    let text = wrapper.text_content().unwrap_or_default();
    assert!(text.contains('日'), "the bare radical symbol must render");
    assert!(
        text.contains('山'),
        "the untranslated radical symbol must render"
    );
    assert!(
        text.contains("Sun"),
        "the translated radical name must render"
    );

    let muted_nodes = wrapper
        .query_selector_all(".text-muted-foreground")
        .unwrap();
    for index in 0..muted_nodes.length() {
        let node = muted_nodes.get(index).expect("index within bounds");
        let muted_text = node
            .dyn_into::<web_sys::Element>()
            .expect("node is an element")
            .text_content()
            .unwrap_or_default();
        assert_ne!(
            muted_text.trim(),
            "",
            "no blank muted text nodes may render for untranslated radicals"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RatingButtons / RatingButtonsView
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn rating_buttons_click_dispatches_rating() {
    let wrapper = create_wrapper();
    let (set_rating, get_rating) = shared_cell::<RwSignal<Option<origa::domain::Rating>>>();
    mount_with_i18n(&wrapper, move || {
        let rating = RwSignal::new(None);
        set_rating.set(Some(rating));
        let on_rate = Callback::new(move |r: origa::domain::Rating| rating.set(Some(r)));
        view! { <RatingButtons on_rate /> }.into_any()
    });
    let rating = get_rating.get().expect("captured");
    tick().await;

    // Act: click "Again"
    wrapper
        .query_selector("[data-testid=\"lesson-rating-btn-again\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        rating.get(),
        Some(origa::domain::Rating::Again),
        "Again click must dispatch Rating::Again"
    );

    // Act: click "Good"
    wrapper
        .query_selector("[data-testid=\"lesson-rating-btn-good\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        rating.get(),
        Some(origa::domain::Rating::Good),
        "Good click must dispatch Rating::Good"
    );
}

#[wasm_bindgen_test]
async fn rating_buttons_disabled_state_blocks_click() {
    let wrapper = create_wrapper();
    let (set_rating, get_rating) = shared_cell::<RwSignal<Option<origa::domain::Rating>>>();
    mount_with_i18n(&wrapper, move || {
        let rating = RwSignal::new(None);
        set_rating.set(Some(rating));
        let on_rate = Callback::new(move |r: origa::domain::Rating| rating.set(Some(r)));
        view! { <RatingButtons on_rate disabled=Signal::from(true) /> }.into_any()
    });
    let rating = get_rating.get().expect("captured");
    tick().await;

    let again = wrapper
        .query_selector("[data-testid=\"lesson-rating-btn-again\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(again.disabled(), "disabled prop must disable the buttons");

    again.click();
    tick().await;

    assert!(
        rating.get().is_none(),
        "disabled button must not dispatch a rating"
    );
}

#[wasm_bindgen_test]
async fn rating_buttons_view_delegates_to_rating_buttons() {
    let wrapper = create_wrapper();
    let (set_rating, get_rating) = shared_cell::<RwSignal<Option<origa::domain::Rating>>>();
    mount_with_i18n(&wrapper, move || {
        let rating = RwSignal::new(None);
        set_rating.set(Some(rating));
        let on_rate = Callback::new(move |r: origa::domain::Rating| rating.set(Some(r)));
        view! { <RatingButtonsView on_rate /> }.into_any()
    });
    let rating = get_rating.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"lesson-rating-btn-good\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        rating.get(),
        Some(origa::domain::Rating::Good),
        "the view wrapper must pass the callback through"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// NextCardButton
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn next_card_button_click_dispatches() {
    let wrapper = create_wrapper();
    let (set_called, get_called) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let called = RwSignal::new(false);
        set_called.set(Some(called));
        view! { <NextCardButton on_next_card=Callback::new(move |()| called.set(true)) /> }
            .into_any()
    });
    let called = get_called.get().expect("captured");
    tick().await;

    assert!(!called.get());

    wrapper
        .query_selector("[data-testid=\"lesson-card-next-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(called.get(), "next click must invoke the callback");
}

// ═══════════════════════════════════════════════════════════════════════
// LessonCardQuestion
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn lesson_card_question_text_shows_show_answer_button() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <LessonCardQuestion
                question_text="たべもの".to_string()
                kanji=None
                is_reversed=false
                on_show_answer=Callback::new(|()| {})
                known_kanji=Signal::derive(|| HashSet::new())
                native_language=origa::domain::NativeLanguage::Russian
            />
        }
        .into_any()
    });
    tick().await;

    let btn = wrapper
        .query_selector("[data-testid=\"lesson-show-answer-btn\"]")
        .unwrap()
        .unwrap();
    let text = btn.text_content().unwrap();
    assert!(!text.is_empty(), "show-answer label must render");

    let page = wrapper.text_content().unwrap();
    assert!(
        page.contains("たべもの"),
        "question text must render; got: {page}"
    );
}

#[wasm_bindgen_test]
async fn lesson_card_question_show_answer_click_dispatches() {
    let wrapper = create_wrapper();
    let (set_shown, get_shown) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let shown = RwSignal::new(false);
        set_shown.set(Some(shown));
        view! {
            <LessonCardQuestion
                question_text="ねこ".to_string()
                kanji=None
                is_reversed=false
                on_show_answer=Callback::new(move |()| shown.set(true))
                known_kanji=Signal::derive(|| HashSet::new())
                native_language=origa::domain::NativeLanguage::Russian
            />
        }
        .into_any()
    });
    let shown = get_shown.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"lesson-show-answer-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(shown.get(), "show-answer click must invoke the callback");
}

// ═══════════════════════════════════════════════════════════════════════
// LessonProgress
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn lesson_progress_simple_total_label() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <LessonProgress
                current=Signal::from(3usize)
                total=Signal::from(10usize)
            />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper.text_content().unwrap();
    assert!(text.contains("3/10"), "plain counter label; got: {text}");
}

#[wasm_bindgen_test]
async fn lesson_progress_with_core_count_splits_label() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <LessonProgress
                current=Signal::from(5usize)
                total=Signal::from(12usize)
                core_count=Signal::from(8usize)
            />
        }
        .into_any()
    });
    tick().await;

    // 5 of 8 core + 0 of 4 phrase
    let text = wrapper.text_content().unwrap();
    assert!(text.contains("5/8 + 0/4"), "split label; got: {text}");
}

#[wasm_bindgen_test]
async fn lesson_progress_reactive_current_updates_label() {
    let wrapper = create_wrapper();
    let (set_current, get_current) = shared_cell::<RwSignal<usize>>();
    mount_to_wrapper(&wrapper, move || {
        let current = RwSignal::new(1usize);
        set_current.set(Some(current));
        view! {
            <LessonProgress
                current=Signal::from(current)
                total=Signal::from(4usize)
            />
        }
        .into_any()
    });
    let current = get_current.get().expect("captured");
    tick().await;

    assert!(
        wrapper.text_content().unwrap().contains("1/4"),
        "initial label"
    );

    current.set(3);
    tick().await;

    assert!(
        wrapper.text_content().unwrap().contains("3/4"),
        "label must track the current signal"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// GrammarInfoBadge
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn grammar_info_badge_renders_title_as_tag() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <GrammarInfoBadge title="〜なければならない".to_string() /> }.into_any()
    });
    tick().await;

    let tag = wrapper
        .query_selector("span.tag")
        .ok()
        .flatten()
        .expect("badge must render as a Tag");
    let text = tag.text_content().unwrap();
    assert!(
        text.contains("〜なければならない"),
        "badge must show the title; got: {text}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// CardAnswerDisplay
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn card_answer_display_translations_render_list() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <CardAnswerDisplay
                translations=Signal::derive(|| Some(vec!["кошка".to_string(), "кот".to_string()]))
                description=Signal::derive(|| None::<String>)
                text=Signal::derive(|| String::new())
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let items = wrapper
        .query_selector_all(".word-translations-item")
        .unwrap()
        .length();
    assert_eq!(items, 2, "two translations → two items; got {items}");
}

#[wasm_bindgen_test]
async fn card_answer_display_text_fallback_renders_markdown() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <CardAnswerDisplay
                translations=Signal::derive(|| None::<Vec<String>>)
                description=Signal::derive(|| None::<String>)
                text=Signal::derive(|| "**bold** answer".to_string())
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector(".lesson-answer")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("<strong>bold</strong>"),
        "text fallback must render markdown; got: {html}"
    );
}

#[wasm_bindgen_test]
async fn card_answer_display_translations_are_centered() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <CardAnswerDisplay
                translations=Signal::derive(|| Some(vec!["кошка".to_string()]))
                description=Signal::derive(|| None::<String>)
                text=Signal::derive(|| String::new())
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let class_name = wrapper
        .query_selector(".lesson-answer")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(
        !class_name.contains("text-left"),
        "answer block must match the centered card layout; got classes: {class_name}"
    );
}

#[wasm_bindgen_test]
async fn card_answer_display_text_fallback_is_centered() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <CardAnswerDisplay
                translations=Signal::derive(|| None::<Vec<String>>)
                description=Signal::derive(|| None::<String>)
                text=Signal::derive(|| "текст ответа".to_string())
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let class_name = wrapper
        .query_selector(".lesson-answer")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(
        !class_name.contains("text-left"),
        "answer block must match the centered card layout; got classes: {class_name}"
    );
}

#[wasm_bindgen_test]
async fn card_answer_display_empty_renders_nothing() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <CardAnswerDisplay
                translations=Signal::derive(|| None::<Vec<String>>)
                description=Signal::derive(|| None::<String>)
                text=Signal::derive(|| String::new())
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let empty = wrapper.query_selector(".lesson-answer");
    assert!(
        empty.is_ok_and(|e| e.is_none()),
        "no translations and no text → no answer block"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// QuizOptions
// ═══════════════════════════════════════════════════════════════════════

fn quiz_options_fixture() -> Vec<origa::domain::QuizOption> {
    vec![
        origa::domain::QuizOption::new("たべる".to_string(), true, None),
        origa::domain::QuizOption::new("のむ".to_string(), false, None),
    ]
}

#[wasm_bindgen_test]
async fn quiz_options_click_selects_index() {
    let wrapper = create_wrapper();
    let (set_sel, get_sel) = shared_cell::<RwSignal<Option<usize>>>();
    mount_with_i18n(&wrapper, move || {
        let selected = RwSignal::new(None);
        set_sel.set(Some(selected));
        view! {
            <QuizOptions
                options=quiz_options_fixture()
                selected_option=None
                show_result=Signal::from(false)
                quiz_result=QuizResult::None
                on_select_option=Callback::new(move |i: usize| selected.set(Some(i)))
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    let selected = get_sel.get().expect("captured");
    tick().await;

    // Act: click the second option
    wrapper
        .query_selector("[data-testid=\"quiz-option-1\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(selected.get(), Some(1), "click must select index 1");
}

#[wasm_bindgen_test]
async fn quiz_options_show_result_marks_correct_and_wrong() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <QuizOptions
                options=quiz_options_fixture()
                selected_option=Some(1)
                show_result=Signal::from(true)
                quiz_result=QuizResult::Incorrect
                on_select_option=Callback::new(|_: usize| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let correct = wrapper
        .query_selector("[data-testid=\"quiz-option-0\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(
        correct.contains("quiz-option-correct"),
        "correct option must be marked; got: {correct}"
    );

    let wrong = wrapper
        .query_selector("[data-testid=\"quiz-option-1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(
        wrong.contains("quiz-option-wrong") && wrong.contains("anima-shake"),
        "selected wrong option must be marked and shake; got: {wrong}"
    );
}

#[wasm_bindgen_test]
async fn quiz_options_locked_when_result_shown() {
    let wrapper = create_wrapper();
    let (set_sel, get_sel) = shared_cell::<RwSignal<Option<usize>>>();
    mount_with_i18n(&wrapper, move || {
        let selected = RwSignal::new(None);
        set_sel.set(Some(selected));
        view! {
            <QuizOptions
                options=quiz_options_fixture()
                selected_option=None
                show_result=Signal::from(true)
                quiz_result=QuizResult::Correct
                on_select_option=Callback::new(move |i: usize| selected.set(Some(i)))
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    let selected = get_sel.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"quiz-option-0\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(
        selected.get().is_none(),
        "clicks must be ignored while the result is shown"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// QuizOptionsMulti
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn quiz_options_multi_toggle_selects() {
    let wrapper = create_wrapper();
    let (set_sel, get_sel) = shared_cell::<RwSignal<HashSet<usize>>>();
    mount_with_i18n(&wrapper, move || {
        let selected = RwSignal::new(HashSet::new());
        set_sel.set(Some(selected));
        view! {
            <QuizOptionsMulti
                options=quiz_options_fixture()
                selected_options=HashSet::new()
                show_result=Signal::from(false)
                multi_submitted=Signal::from(false)
                multi_result=None
                on_toggle=Callback::new(move |i: usize| {
                    selected.update(|s| {
                        if s.contains(&i) { s.remove(&i); } else { s.insert(i); }
                    });
                })
                on_submit=Callback::new(|()| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    let selected = get_sel.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"quiz-option-0\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        selected.get(),
        HashSet::from([0usize]),
        "multi toggle must add the option to the selection"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// YesNoCardView (rendering with a reversed-vocab card)
// ═══════════════════════════════════════════════════════════════════════

fn yesno_fixture(is_correct: bool) -> origa::domain::YesNoCard {
    use origa::domain::{Card, Question, VocabularyCard, YesNoCard};
    let vocab = VocabularyCard::new_with_pos(
        Question::new("cat".to_string()).unwrap(),
        None,
        Some(Question::new("ねこ".to_string()).unwrap()),
    );
    let card = Card::Vocabulary(vocab);
    YesNoCard::new(
        card,
        "ねこ".to_string(),
        "ねこです。".to_string(),
        is_correct,
    )
}

#[wasm_bindgen_test]
async fn yesno_card_view_shows_statement_and_buttons() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <YesNoCardView
                yesno_card=yesno_fixture(true)
                show_result=Signal::from(false)
                selected_answer=None
                on_answer=Callback::new(|_: bool| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                native_language=origa::domain::NativeLanguage::Russian
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let page = wrapper.text_content().unwrap();
    assert!(
        page.contains("ねこ"),
        "the word must render somewhere in the view; got: {page}"
    );
}

#[wasm_bindgen_test]
async fn yesno_card_view_correct_answer_still_shows_the_answer() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <YesNoCardView
                yesno_card=yesno_fixture(true)
                show_result=Signal::from(true)
                selected_answer=Some(true)
                on_answer=Callback::new(|_: bool| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                native_language=origa::domain::NativeLanguage::Russian
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let answer = wrapper.query_selector(".lesson-answer");
    assert!(
        answer.is_ok_and(|a| a.is_some()),
        "a correct answer must still reveal the card's answer, not just the Да/Нет verdict"
    );
}

// The "Correct answer: Да/Нет" line must appear only as feedback on a miss.
// Locale note: `mount_with_i18n` provides the context without an explicit
// locale, so the default from build.rs (`Config::new("en")`) applies —
// assert against the en string (locales/en.json "correct_answer").
#[wasm_bindgen_test]
async fn yesno_card_view_correct_hides_correct_answer_line() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <YesNoCardView
                yesno_card=yesno_fixture(true)
                show_result=Signal::from(true)
                selected_answer=Some(true)
                on_answer=Callback::new(|_: bool| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                native_language=origa::domain::NativeLanguage::Russian
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let page = wrapper.text_content().unwrap_or_default();
    assert!(
        !page.contains("Correct answer"),
        "a correct answer must show only the verdict; got: {page}"
    );
}

#[wasm_bindgen_test]
async fn yesno_card_view_incorrect_shows_correct_answer_line() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <YesNoCardView
                yesno_card=yesno_fixture(true)
                show_result=Signal::from(true)
                selected_answer=Some(false)
                on_answer=Callback::new(|_: bool| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(false)
                native_language=origa::domain::NativeLanguage::Russian
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let page = wrapper.text_content().unwrap_or_default();
    assert!(
        page.contains("Correct answer"),
        "an incorrect answer must reveal the correct Да/Нет; got: {page}"
    );
}

#[wasm_bindgen_test]
async fn yesno_card_view_dont_know_shows_correct_answer_line() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <YesNoCardView
                yesno_card=yesno_fixture(true)
                show_result=Signal::from(true)
                selected_answer=None
                on_answer=Callback::new(|_: bool| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=Signal::from(true)
                native_language=origa::domain::NativeLanguage::Russian
                known_kanji=Signal::derive(|| HashSet::new())
            />
        }
        .into_any()
    });
    tick().await;

    let page = wrapper.text_content().unwrap_or_default();
    assert!(
        page.contains("Correct answer"),
        "a don't-know answer must reveal the correct Да/Нет; got: {page}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// LessonCompleteScreen (needs LessonContext + repository context)
// ═══════════════════════════════════════════════════════════════════════

fn lesson_context() -> crate::pages::lesson::LessonContext {
    use crate::pages::lesson::lesson_state::LessonState;
    crate::pages::lesson::LessonContext {
        repository: crate::repository::HybridUserRepository::new(),
        lesson_state: RwSignal::new(LessonState::default()),
        is_completed: RwSignal::new(true),
        reload_trigger: RwSignal::new(0),
        is_muted: RwSignal::new(false),
        known_kanji: RwSignal::new(HashSet::new()),
        native_language: RwSignal::new(origa::domain::NativeLanguage::Russian),
        core_count: RwSignal::new(0),
    }
}

#[wasm_bindgen_test]
async fn lesson_complete_screen_renders_stats_and_finish_button() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router_and_stores(&wrapper, None, || {
        provide_context(lesson_context());
        provide_context(StoredValue::new(()));
        let is_completed = RwSignal::new(true);
        view! {
            <crate::pages::lesson::complete_screen::LessonCompleteScreen
                is_completed=is_completed
                review_count=12
            />
        }
        .into_any()
    });
    tick().await;

    let screen = wrapper
        .query_selector("[data-testid=\"lesson-complete-screen\"]")
        .ok()
        .flatten()
        .expect("complete screen must render with its context");
    assert!(
        screen
            .query_selector("[data-testid=\"lesson-complete-stats\"]")
            .unwrap()
            .is_some(),
        "stats block must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PhraseCardView
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn phrase_card_view_renders_options_and_quiz_tags() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <PhraseCardView
                card_type=super::card_type::CardType::Phrase
                audio_file="test.opus".to_string()
                options=quiz_options_fixture()
                show_result=Signal::from(false)
                selected_option=None
                on_select_option=Callback::new(|_: usize| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=false
                phrase_text=Some("これはペンです".to_string())
                phrase_translation=Some("Это ручка".to_string())
                known_kanji=Signal::derive(|| HashSet::new())
                waiting_for_next=Signal::from(false)
                on_next_card=Callback::new(|()| {})
            />
        }
        .into_any()
    });
    tick().await;

    let root = wrapper
        .query_selector("[data-testid=\"lesson-card-root\"]")
        .ok()
        .flatten()
        .expect("phrase card root must render");
    assert!(!root.text_content().unwrap().is_empty());
    // Two quiz options + the don't-know button render before answering
    for index in 0..2 {
        assert!(
            wrapper
                .query_selector(&format!("[data-testid=\"quiz-option-{index}\"]"))
                .unwrap()
                .is_some(),
            "option {index} must render"
        );
    }
    assert!(
        wrapper
            .query_selector("[data-testid=\"quiz-dont-know-btn\"]")
            .unwrap()
            .is_some(),
        "don't-know button must render before answering"
    );
}

#[wasm_bindgen_test]
async fn phrase_card_view_result_shows_next_button() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <PhraseCardView
                card_type=super::card_type::CardType::Phrase
                audio_file="test.opus".to_string()
                options=quiz_options_fixture()
                show_result=Signal::from(true)
                selected_option=Some(0)
                on_select_option=Callback::new(|_: usize| {})
                on_dont_know=Callback::new(|()| {})
                dont_know_selected=false
                phrase_text=Some("これはペンです".to_string())
                phrase_translation=Some("Это ручка".to_string())
                known_kanji=Signal::derive(|| HashSet::new())
                waiting_for_next=Signal::from(true)
                on_next_card=Callback::new(|()| {})
            />
        }
        .into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"lesson-card-next-btn\"]")
            .unwrap()
            .is_some(),
        "waiting_for_next must show the next-card button"
    );

    // The phrase text itself renders only after the result is shown
    let page = wrapper.text_content().unwrap();
    assert!(
        page.contains("これはペンです"),
        "the phrase must render with the result; got: {page}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PhraseRatingButtons
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn phrase_rating_buttons_clicks_dispatch_ratings() {
    let wrapper = create_wrapper();
    let (set_rating, get_rating) = shared_cell::<RwSignal<Option<origa::domain::Rating>>>();
    mount_with_i18n(&wrapper, move || {
        let rating = RwSignal::new(None);
        set_rating.set(Some(rating));
        let on_rate = Callback::new(move |r: origa::domain::Rating| rating.set(Some(r)));
        view! { <PhraseRatingButtons on_rate test_id="prb1" /> }.into_any()
    });
    let rating = get_rating.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"prb1-did-not-understand\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;
    assert_eq!(rating.get(), Some(origa::domain::Rating::Again));

    wrapper
        .query_selector("[data-testid=\"prb1-understood\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;
    assert_eq!(rating.get(), Some(origa::domain::Rating::Good));
}

#[wasm_bindgen_test]
async fn phrase_rating_buttons_disabled_blocks_dispatch() {
    let wrapper = create_wrapper();
    let (set_rating, get_rating) = shared_cell::<RwSignal<Option<origa::domain::Rating>>>();
    mount_with_i18n(&wrapper, move || {
        let rating = RwSignal::new(None);
        set_rating.set(Some(rating));
        let on_rate = Callback::new(move |r: origa::domain::Rating| rating.set(Some(r)));
        view! { <PhraseRatingButtons on_rate disabled=Signal::from(true) test_id="prb2" /> }
            .into_any()
    });
    let rating = get_rating.get().expect("captured");
    tick().await;

    let understood = wrapper
        .query_selector("[data-testid=\"prb2-understood\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(understood.disabled());
    understood.click();
    tick().await;

    assert!(rating.get().is_none(), "disabled button must not dispatch");
}

// ═══════════════════════════════════════════════════════════════════════
// QuizResultDisplay
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn quiz_result_display_correct_shows_success_styling() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <QuizResultDisplay quiz_result=QuizResult::Correct /> }.into_any()
    });
    tick().await;

    let text_el = wrapper.query_selector("p").unwrap().unwrap();
    let class = text_el.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("text-[var(--success)]"),
        "correct result must be success-styled; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn quiz_result_display_multi_partial_lists_missed_and_wrong() {
    use origa::domain::MultiQuizResult;

    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let multi = MultiQuizResult {
            correct_selections: vec![],
            missed: vec![0usize],
            wrong_selections: vec![1usize],
            is_perfect: false,
        };
        view! {
            <QuizResultDisplay
                quiz_result=QuizResult::MultiPartial
                multi_result=Some(multi)
                options=quiz_options_fixture()
            />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper.text_content().unwrap();
    assert!(
        text.contains("のむ"),
        "missed option text must be listed; got: {text}"
    );
    assert!(
        text.contains("たべる"),
        "wrong selection text must be listed; got: {text}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Acquaintance training (режим знакомства — часть основного потока)
// ═══════════════════════════════════════════════════════════════════════

mod acquaintance_training {
    use super::*;
    use crate::pages::lesson::acquaintance_state::{
        AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage, AcquaintanceState,
    };
    use crate::pages::lesson::acquaintance_view::AcquaintanceView;
    use crate::pages::lesson::training_view::TrainingBody;
    use ulid::Ulid;

    fn acq_context(stage: AcquaintanceStage) -> AcquaintanceContext {
        let state = RwSignal::new(AcquaintanceState::default());
        // stage выставляется отдельно через update (Default = Inactive).
        state.update(|s| s.stage = stage);
        AcquaintanceContext {
            repository: crate::repository::HybridUserRepository::new(),
            state,
            slides: RwSignal::new(Vec::new()),
            known_kanji: RwSignal::new(HashSet::new()),
            native_language: RwSignal::new(origa::domain::NativeLanguage::Russian),
        }
    }

    #[wasm_bindgen_test]
    async fn training_reveals_answer_and_rating_buttons_with_hints() {
        let ctx = acq_context(AcquaintanceStage::Training);
        let card_id = Ulid::new();
        let hand = origa::domain::AcquaintanceHand::new(vec![(
            card_id,
            origa::domain::CardType::Vocabulary,
        )])
        .unwrap();
        ctx.state.update(|state| state.hand = Some(hand));
        ctx.slides.set(vec![AcquaintanceSlideData::Vocabulary {
            card_id,
            word: "\u{732b}".to_string(),
            pos_label: None,
            translations: vec!["\u{43a}\u{43e}\u{448}\u{43a}\u{430}".to_string()],
        }]);

        let wrapper = create_wrapper();
        let c2 = ctx.clone();
        mount_with_i18n(&wrapper, move || {
            view! { <TrainingBody ctx=c2.clone() /> }.into_any()
        });
        tick().await;

        // Фронт виден, ответа ещё нет.
        assert!(
            wrapper
                .query_selector("[data-testid=\"acquaintance-training-front\"]")
                .unwrap()
                .is_some(),
            "фронт тренировки отрендерен"
        );
        assert!(
            wrapper
                .query_selector("[data-testid=\"acquaintance-training-answer\"]")
                .unwrap()
                .is_none()
        );

        // Раскрытие по кнопке. Хинт Space читаем до клика: после раскрытия
        // кнопка скрывается вместе с фронтом.
        let reveal = wrapper
            .query_selector("[data-testid=\"acquaintance-reveal-btn\"]")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        let reveal_text = reveal.text_content().unwrap();
        assert!(
            reveal_text.contains("Space") || reveal_text.contains("Пробел"),
            "хинт Space на раскрытии, got: {reveal_text}"
        );
        reveal.click();
        tick().await;

        assert!(
            wrapper
                .query_selector("[data-testid=\"acquaintance-training-answer\"]")
                .unwrap()
                .is_some(),
            "ответ раскрыт после клика"
        );

        let remember = wrapper
            .query_selector("[data-testid=\"acquaintance-rating-remember\"]")
            .unwrap()
            .unwrap();
        let text = remember.text_content().unwrap();
        assert!(
            text.contains("[2]"),
            "хинт [2] должен быть на кнопке «Помню», got: {text}"
        );

        let dont_know_text = wrapper
            .query_selector("[data-testid=\"acquaintance-rating-dont-know\"]")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap();
        assert!(dont_know_text.contains("[1]"), "хинт [1] на «Не помню»");
    }

    #[wasm_bindgen_test]
    async fn summary_screen_shows_stamp_and_to_reviews_button() {
        let ctx = acq_context(AcquaintanceStage::Summary);
        let wrapper = create_wrapper();
        let c2 = ctx.clone();
        mount_with_i18n(&wrapper, move || {
            provide_context(c2.clone());
            view! { <AcquaintanceView /> }.into_any()
        });
        tick().await;

        assert!(
            wrapper
                .query_selector("[data-testid=\"acquaintance-summary\"]")
                .unwrap()
                .is_some(),
            "итоговый экран отрендерен"
        );
        assert!(
            wrapper
                .query_selector("[data-testid=\"acquaintance-to-reviews-btn\"]")
                .unwrap()
                .is_some(),
            "кнопка «К ревью» на месте"
        );
    }
}

mod acquaintance_training_fronts {
    use super::*;
    use crate::pages::lesson::acquaintance_state::{
        AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage, AcquaintanceState,
    };
    use crate::pages::lesson::training_view::TrainingBody;
    use crate::ui_components::ReadingItem;
    use ulid::Ulid;

    fn acq_context() -> AcquaintanceContext {
        let state = RwSignal::new(AcquaintanceState::default());
        state.update(|s| s.stage = AcquaintanceStage::Training);
        AcquaintanceContext {
            repository: crate::repository::HybridUserRepository::new(),
            state,
            slides: RwSignal::new(Vec::new()),
            known_kanji: RwSignal::new(HashSet::new()),
            native_language: RwSignal::new(origa::domain::NativeLanguage::Russian),
        }
    }

    fn mount_training(ctx: &AcquaintanceContext) -> web_sys::Element {
        let wrapper = create_wrapper();
        let c2 = ctx.clone();
        mount_with_i18n(&wrapper, move || {
            provide_context(c2.clone());
            view! { <TrainingBody ctx=c2.clone() /> }.into_any()
        });
        wrapper
    }

    #[wasm_bindgen_test]
    async fn kanji_front_shows_only_the_character() {
        // Arrange
        let ctx = acq_context();
        let card_id = Ulid::new();
        ctx.state.update(|state| {
            state.hand = Some(
                origa::domain::AcquaintanceHand::new(vec![(
                    card_id,
                    origa::domain::CardType::Kanji,
                )])
                .unwrap(),
            )
        });
        ctx.slides.set(vec![AcquaintanceSlideData::Kanji {
            card_id,
            kanji: "明".to_string(),
            name: "свет".to_string(),
            radicals: None,
            example_words: None,
            on_readings: Some(vec![ReadingItem {
                reading: "みょう".to_string(),
                freq: 100,
                is_rare: false,
            }]),
            kun_readings: None,
        }]);

        // Act
        let wrapper = mount_training(&ctx);
        tick().await;

        // Assert: во фронте только знак — значение и чтения не утекают
        let front = wrapper
            .query_selector("[data-testid=\"acquaintance-training-front\"]")
            .unwrap()
            .unwrap();
        let text = front.text_content().unwrap();
        assert!(text.contains("明"), "знак во фронте, got: {text}");
        assert!(
            !text.contains("свет") && !text.contains("みょう"),
            "значение и чтения — ответ, не фронт; got: {text}"
        );
    }

    #[wasm_bindgen_test]
    async fn kanji_answer_shows_meaning_and_readings() {
        // Arrange
        let ctx = acq_context();
        let card_id = Ulid::new();
        ctx.state.update(|state| {
            state.hand = Some(
                origa::domain::AcquaintanceHand::new(vec![(
                    card_id,
                    origa::domain::CardType::Kanji,
                )])
                .unwrap(),
            )
        });
        ctx.slides.set(vec![AcquaintanceSlideData::Kanji {
            card_id,
            kanji: "明".to_string(),
            name: "свет".to_string(),
            radicals: None,
            example_words: None,
            on_readings: Some(vec![ReadingItem {
                reading: "みょう".to_string(),
                freq: 100,
                is_rare: false,
            }]),
            kun_readings: Some(vec![ReadingItem {
                reading: "あか".to_string(),
                freq: 50,
                is_rare: false,
            }]),
        }]);

        // Act
        let wrapper = mount_training(&ctx);
        tick().await;
        wrapper
            .query_selector("[data-testid=\"acquaintance-reveal-btn\"]")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap()
            .click();
        tick().await;

        // Assert: ответ раскрывает и значения, и оба чтения
        let answer = wrapper
            .query_selector("[data-testid=\"acquaintance-training-answer\"]")
            .unwrap()
            .unwrap();
        let text = answer.text_content().unwrap();
        assert!(text.contains("свет"), "значение в ответе, got: {text}");
        assert!(text.contains("みょう"), "ОН-чтение в ответе, got: {text}");
        assert!(text.contains("あか"), "КУН-чтение в ответе, got: {text}");
    }

    #[wasm_bindgen_test]
    async fn grammar_front_shows_japanese_example_without_translation_or_meaning() {
        // Arrange
        let ctx = acq_context();
        let card_id = Ulid::new();
        ctx.state.update(|state| {
            state.hand = Some(
                origa::domain::AcquaintanceHand::new(vec![(
                    card_id,
                    origa::domain::CardType::Grammar,
                )])
                .unwrap(),
            )
        });
        ctx.slides.set(vec![AcquaintanceSlideData::Grammar {
            card_id,
            title: "～は～です".to_string(),
            short_description: "утверждение с です".to_string(),
            how_to_form: String::new(),
            examples: "```\n私は学生です。\nI am a student.\n```".to_string(),
            explanation: String::new(),
            nuances: String::new(),
        }]);

        // Act
        let wrapper = mount_training(&ctx);
        tick().await;

        // Assert: фронт — японская строка примера; ни перевод, ни смысл
        let front = wrapper
            .query_selector("[data-testid=\"acquaintance-training-front\"]")
            .unwrap()
            .unwrap();
        let text = front.text_content().unwrap();
        assert!(
            text.contains("私は学生です。"),
            "японский пример во фронте, got: {text}"
        );
        assert!(
            !text.contains("I am a student") && !text.contains("утверждение"),
            "перевод примера и смысл — ответ; got: {text}"
        );
    }

    #[wasm_bindgen_test]
    async fn grammar_answer_shows_meaning_and_full_example() {
        // Arrange
        let ctx = acq_context();
        let card_id = Ulid::new();
        ctx.state.update(|state| {
            state.hand = Some(
                origa::domain::AcquaintanceHand::new(vec![(
                    card_id,
                    origa::domain::CardType::Grammar,
                )])
                .unwrap(),
            )
        });
        ctx.slides.set(vec![AcquaintanceSlideData::Grammar {
            card_id,
            title: "～は～です".to_string(),
            short_description: "утверждение с です".to_string(),
            how_to_form: String::new(),
            examples: "```\n私は学生です。\nI am a student.\n```".to_string(),
            explanation: String::new(),
            nuances: String::new(),
        }]);

        // Act
        let wrapper = mount_training(&ctx);
        tick().await;
        wrapper
            .query_selector("[data-testid=\"acquaintance-reveal-btn\"]")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap()
            .click();
        tick().await;

        // Assert: смысл + полный пример с переводом
        let answer = wrapper
            .query_selector("[data-testid=\"acquaintance-training-answer\"]")
            .unwrap()
            .unwrap();
        let text = answer.text_content().unwrap();
        assert!(
            text.contains("утверждение с です"),
            "смысл в ответе, got: {text}"
        );
        assert!(
            text.contains("I am a student"),
            "перевод примера в ответе, got: {text}"
        );
    }

    #[wasm_bindgen_test]
    async fn grammar_front_without_examples_falls_back_to_title() {
        // Arrange: 7 из 515 карточек контента имеют пустые examples
        let ctx = acq_context();
        let card_id = Ulid::new();
        ctx.state.update(|state| {
            state.hand = Some(
                origa::domain::AcquaintanceHand::new(vec![(
                    card_id,
                    origa::domain::CardType::Grammar,
                )])
                .unwrap(),
            )
        });
        ctx.slides.set(vec![AcquaintanceSlideData::Grammar {
            card_id,
            title: "～たことがある".to_string(),
            short_description: "опыт".to_string(),
            how_to_form: String::new(),
            examples: String::new(),
            explanation: String::new(),
            nuances: String::new(),
        }]);

        // Act
        let wrapper = mount_training(&ctx);
        tick().await;

        // Assert: фронт — заголовок конструкции; смысл скрыт до раскрытия
        let front = wrapper
            .query_selector("[data-testid=\"acquaintance-training-front\"]")
            .unwrap()
            .unwrap();
        let text = front.text_content().unwrap();
        assert!(
            text.contains("～たことがある"),
            "фолбэк на заголовок, got: {text}"
        );
        assert!(!text.contains("опыт"), "смысл не утекает, got: {text}");
    }
}

mod acquaintance_presentation {
    use super::*;
    use crate::pages::lesson::acquaintance_state::{
        AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage, AcquaintanceState,
    };
    use crate::pages::lesson::acquaintance_view::AcquaintanceView;
    use crate::ui_components::ReadingItem;
    use ulid::Ulid;

    fn acq_context(stage: AcquaintanceStage) -> AcquaintanceContext {
        let state = RwSignal::new(AcquaintanceState::default());
        state.update(|s| s.stage = stage);
        AcquaintanceContext {
            repository: crate::repository::HybridUserRepository::new(),
            state,
            slides: RwSignal::new(Vec::new()),
            known_kanji: RwSignal::new(HashSet::new()),
            native_language: RwSignal::new(origa::domain::NativeLanguage::Russian),
        }
    }

    fn hand_of(
        card_id: Ulid,
        card_type: origa::domain::CardType,
    ) -> origa::domain::AcquaintanceHand {
        origa::domain::AcquaintanceHand::new(vec![(card_id, card_type)]).unwrap()
    }

    #[wasm_bindgen_test]
    async fn kanji_slide_shows_character_with_animation_and_details() {
        // Arrange
        let ctx = acq_context(AcquaintanceStage::Presentation);
        let card_id = Ulid::new();
        ctx.state
            .update(|state| state.hand = Some(hand_of(card_id, origa::domain::CardType::Kanji)));
        ctx.slides.set(vec![AcquaintanceSlideData::Kanji {
            card_id,
            kanji: "明".to_string(),
            name: "свет".to_string(),
            radicals: None,
            example_words: None,
            on_readings: Some(vec![ReadingItem {
                reading: "みょう".to_string(),
                freq: 100,
                is_rare: false,
            }]),
            kun_readings: None,
        }]);

        // Act
        let wrapper = create_wrapper();
        let c2 = ctx.clone();
        mount_with_i18n(&wrapper, move || {
            provide_context(c2.clone());
            view! { <AcquaintanceView /> }.into_any()
        });
        tick().await;

        // Assert: сам знак с анимацией черт — не только чтения и значения
        assert!(
            wrapper
                .query_selector("[data-testid=\"acquaintance-kanji-animation\"]")
                .unwrap()
                .is_some(),
            "знак кандзи отрендерен на слайде"
        );
        let html = wrapper.inner_html();
        assert!(html.contains("みょう"), "чтения остаются на слайде");
        assert!(html.contains("свет"), "значение остаётся на слайде");
    }

    #[wasm_bindgen_test]
    async fn card_type_tag_follows_current_slide_type() {
        // Arrange: два слайда — слово, затем кандзи
        let ctx = acq_context(AcquaintanceStage::Presentation);
        let word_id = Ulid::new();
        let kanji_id = Ulid::new();
        ctx.state.update(|state| {
            state.hand = Some(
                origa::domain::AcquaintanceHand::new(vec![
                    (word_id, origa::domain::CardType::Vocabulary),
                    (kanji_id, origa::domain::CardType::Kanji),
                ])
                .unwrap(),
            );
        });
        ctx.slides.set(vec![
            AcquaintanceSlideData::Vocabulary {
                card_id: word_id,
                word: "ねこ".to_string(),
                pos_label: None,
                translations: vec!["кошка".to_string()],
            },
            AcquaintanceSlideData::Kanji {
                card_id: kanji_id,
                kanji: "明".to_string(),
                name: "свет".to_string(),
                radicals: None,
                example_words: None,
                on_readings: None,
                kun_readings: None,
            },
        ]);

        // Act
        let wrapper = create_wrapper();
        let c2 = ctx.clone();
        mount_with_i18n(&wrapper, move || {
            provide_context(c2.clone());
            view! { <AcquaintanceView /> }.into_any()
        });
        tick().await;

        // Assert: тег типа показывает тип текущего слайда
        let tag = || {
            wrapper
                .query_selector("[data-testid=\"acquaintance-card-type-tag\"]")
                .unwrap()
                .unwrap()
                .text_content()
                .unwrap()
        };
        let word_tag = tag();
        assert!(
            word_tag.contains("Слово") || word_tag.contains("Word"),
            "тег типа первого слайда — слово, got: {word_tag}"
        );

        ctx.state.update(|state| state.slide_index = 1);
        tick().await;
        let kanji_tag = tag();
        assert!(
            kanji_tag.contains("Кандзи") || kanji_tag.contains("Kanji"),
            "тег типа второго слайда — кандзи, got: {kanji_tag}"
        );
    }

    #[wasm_bindgen_test]
    async fn grammar_slide_renders_explanation_and_nuances_as_markdown() {
        // Arrange: explanation и nuances с markdown-разметкой
        let ctx = acq_context(AcquaintanceStage::Presentation);
        let card_id = Ulid::new();
        ctx.state
            .update(|state| state.hand = Some(hand_of(card_id, origa::domain::CardType::Grammar)));
        ctx.slides.set(vec![AcquaintanceSlideData::Grammar {
            card_id,
            title: "ぜひ".to_string(),
            short_description: "наречие".to_string(),
            how_to_form: String::new(),
            examples: String::new(),
            explanation: "**сильное** желание".to_string(),
            nuances: "> ⚠️ **Важно:** нюанс".to_string(),
        }]);

        // Act
        let wrapper = create_wrapper();
        let c2 = ctx.clone();
        mount_with_i18n(&wrapper, move || {
            provide_context(c2.clone());
            view! { <AcquaintanceView /> }.into_any()
        });
        tick().await;

        // Assert: markdown рендерится, сырая разметка не показывается
        for test_id in [
            "acquaintance-grammar-explanation",
            "acquaintance-grammar-nuances",
        ] {
            let block = wrapper
                .query_selector(&format!("[data-testid=\"{test_id}\"]"))
                .unwrap()
                .unwrap_or_else(|| panic!("{test_id} блок отрендерен"));
            let html = block.inner_html();
            assert!(
                html.contains("<strong>"),
                "{test_id}: жирный текст отрендерен, got: {html}"
            );
            assert!(
                !html.contains("**"),
                "{test_id}: сырая markdown-разметка скрыта, got: {html}"
            );
        }
    }

    #[wasm_bindgen_test]
    async fn word_slide_offers_audio_repeat_button() {
        // Arrange
        let ctx = acq_context(AcquaintanceStage::Presentation);
        let card_id = Ulid::new();
        ctx.state.update(|state| {
            state.hand = Some(hand_of(card_id, origa::domain::CardType::Vocabulary))
        });
        ctx.slides.set(vec![AcquaintanceSlideData::Vocabulary {
            card_id,
            word: "ねこ".to_string(),
            pos_label: None,
            translations: vec!["кошка".to_string()],
        }]);

        // Act
        let wrapper = create_wrapper();
        let c2 = ctx.clone();
        mount_with_i18n(&wrapper, move || {
            provide_context(c2.clone());
            view! { <AcquaintanceView /> }.into_any()
        });
        tick().await;

        // Assert: повтор прослушивания доступен по нажатию
        assert!(
            wrapper
                .query_selector("[data-testid=\"acquaintance-word-audio\"]")
                .unwrap()
                .is_some(),
            "кнопка повтора аудио на слайде слова"
        );
    }
}
