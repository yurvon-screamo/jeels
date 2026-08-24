#[cfg(feature = "acquaintance_mode")]
mod acquaintance_state;
#[cfg(feature = "acquaintance_mode")]
mod acquaintance_view;
mod answer_display;
pub mod card_type;
pub(crate) mod complete_screen;
mod content;
mod empty_state_view;
mod grammar_details_expand;
mod grammar_info_badge;
#[cfg(feature = "acquaintance_mode")]
mod hand_progress_strip;
mod header;
mod kanji_card_details;
mod keyboard_handler;
mod lesson_card;
mod lesson_card_answer;
mod lesson_card_container;
mod lesson_card_header;
mod lesson_card_question;
mod lesson_card_renderer;
mod lesson_progress;
mod lesson_state;
#[cfg(all(target_arch = "wasm32", test))]
mod lesson_wasm_tests;
mod na_adjective_helper;
mod next_card_button;
mod on_dont_know;
mod on_quiz_select;
mod on_quiz_submit;
mod on_quiz_toggle;
mod on_rate;
mod on_yesno_select;
pub(crate) mod phrase_card;
pub(crate) mod phrase_rating_buttons;
pub(crate) mod pos_label;
mod quiz_card;
mod quiz_options;
mod quiz_options_multi;
mod quiz_result;
pub(crate) mod quiz_result_display;
mod rating_buttons;
mod rating_buttons_view;
#[cfg(feature = "acquaintance_mode")]
mod training_view;
mod writing_card;
mod yesno_card_view;

pub use content::LessonContent;
pub use lesson_state::LessonContext;

use leptos::prelude::*;

/// Shared Tailwind class for every lesson card view. Pins a stable minimum
/// height in `svh` (small viewport height — stable in both Tauri WebView and
/// dev browser, unlike `dvh` which floats on URL bar collapse) plus `grow` to
/// fill the available `lesson-content` rectangle. See ADR-031.
pub(in crate::pages::lesson) const LESSON_CARD_CLASS: &str =
    "p-4 sm:p-6 min-h-[60svh] sm:min-h-[70svh] flex flex-col grow";

#[component]
pub fn Lesson() -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col py-4" data-testid="lesson-page">
            <div class="min-w-[min(90vw,1280px)] max-w-[min(90vw,1280px)] mx-auto px-1 sm:px-2 flex-1 flex flex-col min-h-0" data-testid="lesson-card">
                <LessonContent />
            </div>
        </div>
    }
}
