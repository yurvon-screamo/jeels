mod content;
mod filters;
mod header;
mod import_set_preview_modal;
mod import_set_preview_modal_handlers;
mod import_set_preview_modal_state;
mod import_slice_budget;
mod set_card;
mod set_word_item;
mod sets_level_group;
mod sets_type_group;
#[cfg(all(target_arch = "wasm32", test))]
mod sets_wasm_tests;
mod types;

pub use content::SetsContent;
pub use header::SetsHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Sets() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="sets-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="sets-card">
                <SetsHeader />
                <SetsContent />
            </CardLayout>
        </PageLayout>
    }
}
