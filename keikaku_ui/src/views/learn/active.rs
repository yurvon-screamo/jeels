use dioxus::prelude::*;

use super::{LearnCardDisplay, LearnProgress};

#[component]
pub fn LearnActive(
    current_card: Option<super::LearnCard>,
    total_cards: usize,
    current_index: usize,
    current_step: super::LearnStep,
    show_furigana: bool,
    native_language: keikaku::domain::value_objects::NativeLanguage,
    on_next: EventHandler<()>,
    on_show_answer: EventHandler<()>,
    on_prev: Option<EventHandler<()>>,
    on_rate: EventHandler<crate::domain::Rating>,
    on_skip: EventHandler<()>,
    on_quit: EventHandler<()>,
) -> Element {
    let progress = {
        let current = current_index;
        let total = total_cards;
        if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    };

    rsx! {
        div { class: "space-y-6",
            LearnCardDisplay {
                card: current_card,
                current_step: current_step.clone(),
                show_furigana,
                native_language,
                on_show_answer,
                on_next,
                on_rate,
            }


            LearnProgress {
                current: current_index + 1,
                total: total_cards,
                progress,
            }
        }
    }
}
