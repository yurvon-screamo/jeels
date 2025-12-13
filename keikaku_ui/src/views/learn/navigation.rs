use dioxus::prelude::*;

#[component]
pub fn LearnNavigation(
    current_index: usize,
    total_cards: usize,
    current_step: super::LearnStep,
    on_next: EventHandler<()>,
    on_prev: Option<EventHandler<()>>,
) -> Element {
    let is_first_card = current_index == 0;
    let can_go_prev = !is_first_card && current_step == super::LearnStep::Answer;

    rsx! {
        div { class: "flex gap-3",
            if let Some(on_prev) = on_prev {
                if can_go_prev {
                    crate::ui::Button {
                        variant: crate::ui::ButtonVariant::Outline,
                        class: Some("flex-1".to_string()),
                        onclick: move |_| on_prev.call(()),
                        disabled: Some(!can_go_prev),
                        "← Назад"
                    }
                }
            }

            if current_step == super::LearnStep::Completed && current_index + 1 < total_cards {
                crate::ui::Button {
                    variant: crate::ui::ButtonVariant::Rainbow,
                    class: Some("flex-1".to_string()),
                    onclick: move |_| on_next.call(()),
                    "Следующая карточка"
                }
            }
        }
    }
}
