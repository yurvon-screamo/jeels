use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, SectionHeader};

#[component]
pub fn CardsHeader(
    total_count: usize,
    due_count: usize,
    on_create_click: EventHandler<()>,
) -> Element {
    rsx! {
        SectionHeader {
            title: "Карточки".to_string(),
            subtitle: Some(
                "Управление карточками для изучения".to_string(),
            ),
            actions: Some(rsx! {
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-auto px-6".to_string()),
                    onclick: move |_| on_create_click.call(()),
                    "+ Создать карточку"
                }
            }),
        }
    }
}
