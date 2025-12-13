use dioxus::prelude::*;

use crate::ui::{Card, SectionHeader};

use super::LearnSettings;

#[component]
pub fn Learn() -> Element {
    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Обучение".to_string(),
                subtitle: Some("Изучайте и повторяйте материал".to_string()),
                actions: None,
            }

            LearnSettings {}
        }
    }
}
