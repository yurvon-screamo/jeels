mod add_grammar_modal;
mod add_grammar_modal_handlers;
mod add_grammar_modal_state;
mod content;
mod grammar_card_item;
mod grammar_detail;
mod grammar_detail_hero_card;
mod grammar_detail_mobile;
mod grammar_practice_session;
pub(crate) mod grammar_warnings;
mod header;
pub(crate) mod nuances_section;
pub(crate) mod related_pattern_list;
mod rule_item;
mod rules_list;

pub use content::GrammarContent;
pub use grammar_detail::GrammarDetail;
pub use header::GrammarHeader;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

#[component]
pub fn Grammar() -> impl IntoView {
    let refresh_trigger = RwSignal::new(0u32);

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="grammar-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="grammar-card">
                <GrammarHeader refresh_trigger=refresh_trigger />
                <GrammarContent refresh_trigger=refresh_trigger />
            </CardLayout>
        </PageLayout>
    }
}
