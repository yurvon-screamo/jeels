//! WASM render tests for the acquaintance presentation phase (S4):
//! `HandProgressStrip`, `AcquaintanceView` action bar inline-confirm flow,
//! word slide content. Mirrors `lesson_wasm_tests` harness conventions.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::pages::lesson::acquaintance_state::{
    AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage, AcquaintanceState,
};
use crate::pages::lesson::acquaintance_view::AcquaintanceView;
use crate::pages::lesson::hand_progress_strip::HandProgressStrip;

fn text_content(root: &web_sys::Element, test_id: &str) -> Option<String> {
    root.query_selector(&format!("[data-testid=\"{test_id}\"]"))
        .ok()?
        .map(|el| el.text_content().unwrap_or_default())
}

fn mount_acquaintance(slides: Vec<AcquaintanceSlideData>, hand_len: usize) -> AcquaintanceContext {
    let state = RwSignal::new(AcquaintanceState {
        stage: AcquaintanceStage::Presentation,
        ..Default::default()
    });
    let ctx = AcquaintanceContext {
        state,
        slides: RwSignal::new(slides),
        known_kanji: RwSignal::new(HashSet::new()),
        native_language: RwSignal::new(origa::domain::NativeLanguage::Russian),
    };
    provide_context(ctx.clone());

    // Рука с hand_len картами: доменная машина нужна только ради длины.
    let entries = (0..hand_len)
        .map(|_| (ulid::Ulid::new(), origa::domain::CardType::Vocabulary))
        .collect();
    if let Ok(hand) = origa::domain::AcquaintanceHand::new(entries) {
        state.update(|s| s.hand = Some(hand));
    }
    ctx
}

#[test]
fn strip_renders_one_cell_per_hand_card() {
    let owner = Owner::new();
    owner.set();
    let (total, progress) = (Signal::derive(|| 3usize), Signal::derive(|| vec![1u8, 3, 0]));
    let _view = mount_to_body(move || {
        view! { <HandProgressStrip total total_progress=progress /> }
    });
    tick();
    let document = leptos::prelude::document();
    let strip = document
        .query_selector("[data-testid=\"acquaintance-strip\"]")
        .unwrap()
        .unwrap();
    assert_eq!(
        strip.children().length(),
        3,
        "ячейка на каждую карту руки"
    );
}

#[test]
fn presentation_bar_confirms_known_via_inline_panel() {
    let owner = Owner::new();
    owner.set();

    let slides = vec![AcquaintanceSlideData::Vocabulary {
        card_id: ulid::Ulid::new(),
        word: "猫".to_string(),
        pos_label: Some("сущ.".to_string()),
        translations: vec!["кошка".to_string()],
    }];
    let ctx = mount_acquaintance(slides, 1);
    let _view = mount_to_body(move || view! { <AcquaintanceView /> });
    tick();

    let document = leptos::prelude::document();
    assert!(
        document
            .query_selector("[data-testid=\"acquaintance-word-slide\"]")
            .is_ok(),
        "слайд слова отрендерен"
    );

    // «Уже знаю» → inline-подтверждение появляется
    let know_btn = document
        .query_selector("[data-testid=\"acquaintance-know-btn\"]")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    know_btn.click();
    tick();
    assert!(
        document
            .query_selector("[data-testid=\"acquaintance-know-confirm-panel\"]")
            .is_ok(),
        "inline-подтверждение открыто"
    );

    // «Нет» закрывает подтверждение
    let cancel_btn = document
        .query_selector("[data-testid=\"acquaintance-know-cancel\"]")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    cancel_btn.click();
    tick();
    assert!(document.query_selector("[data-testid=\"acquaintance-know-confirm-panel\"]").is_err());
}
