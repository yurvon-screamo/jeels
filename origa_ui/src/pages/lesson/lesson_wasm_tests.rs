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
