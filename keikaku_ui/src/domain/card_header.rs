use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, SectionHeader};

#[component]
pub fn CardHeader(total_count: usize, due_count: usize) -> Element {
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
                    onclick: move |_| {},
                    "+ Создать карточку"
                }
            }),
        }
    }
}
