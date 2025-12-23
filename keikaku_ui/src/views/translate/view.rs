use crate::components::app_ui::{Card, Paragraph, SectionHeader};
use crate::components::button::{Button, ButtonVariant};
use crate::components::textarea::{Textarea, TextareaVariant};
use dioxus::prelude::*;

use super::translate::{UseTranslate, use_translate};

#[component]
pub fn Translate() -> Element {
    let translator = use_translate();

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Перевод".to_string(),
                subtitle: "Переводчик предназначен для перевода предложений и текстов, работа с одиночными словами может быть некорректной.",
                actions: None,
            }

            Card { class: Some("grid grid-cols-1 md:grid-cols-3 gap-4".to_string()),
                TranslationInput { translator: translator.clone() }
                TranslationResult { translator }
            }
        }
    }
}

#[component]
fn TranslationInput(mut translator: UseTranslate) -> Element {
    rsx! {
        div { class: "space-y-2",
            Textarea {
                variant: TextareaVariant::Default,
                rows: 8,
                placeholder: "日本語 или русский текст",
                value: (translator.text)(),
                oninput: move |e: FormEvent| translator.text.set(e.value()),
            }

            TranslateButton { translator: translator.clone() }
        }
    }
}

#[component]
fn DirectionButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        Button {
            variant: if active { ButtonVariant::Primary } else { ButtonVariant::Outline },
            class: "w-full text-sm",
            onclick: move |e| onclick.call(e),
            {label}
        }
    }
}

#[component]
fn TranslateButton(mut translator: UseTranslate) -> Element {
    rsx! {
        {
            let mut translator_clone = translator.clone();
            let is_loading = (translator.loading)();
            rsx! {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    onclick: move |_| translator_clone.translate(),
                    {if is_loading { "Перевод..." } else { "Перевести" }}
                }
            }
        }
    }
}

#[component]
fn TranslationResult(mut translator: UseTranslate) -> Element {
    rsx! {
        Card { class: Some("bg-slate-50 border border-slate-100 p-4 space-y-2".to_string()),
            Paragraph { class: Some("text-sm font-semibold text-slate-700".to_string()), "Результат" }
            if let Some(result) = (translator.result)() {
                Paragraph { class: Some("text-base text-slate-700".to_string()), {result} }
            } else {
                Paragraph { class: Some("text-sm text-slate-500".to_string()),
                    "Здесь появится перевод."
                }
            }
        }
    }
}
