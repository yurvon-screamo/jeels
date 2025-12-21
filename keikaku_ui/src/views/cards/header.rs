use dioxus::prelude::*;

use crate::Route;
use crate::components::app_ui::SectionHeader;
use crate::components::button::{Button, ButtonVariant};

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
                Link { to: Route::Learn {},
                    Button { variant: ButtonVariant::Outline, class: "w-auto px-6", "Учиться" }
                }
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-auto px-6",
                    onclick: move |_| on_create_click.call(()),
                    "+ Создать карточку"
                }
            }),
        }
    }
}
