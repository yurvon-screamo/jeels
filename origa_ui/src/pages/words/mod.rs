mod add_word_modal;
mod add_words_preview_modal;
mod add_words_preview_modal_handlers;
mod add_words_preview_modal_state;
pub(crate) mod analyzed_word_item;
mod anki_import_stage;
mod asr_provider;
mod audio_input_stage;
mod audio_live_recorder;
mod content;
mod header;
mod image_input_stage;
mod ocr_device_ai;
mod ocr_file_utils;
mod ocr_processing;
pub(crate) mod vocabulary_card_item;
#[cfg(all(target_arch = "wasm32", test))]
mod words_wasm_tests;

pub use content::WordsContent;
pub use header::WordsHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Words() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="words-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="words-card">
                <WordsHeader refresh_trigger=refresh_trigger />
                <WordsContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
