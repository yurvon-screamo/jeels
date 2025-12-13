use crate::components::{Button, ButtonVariant, Card, Paragraph, SectionHeader, Textarea};
use crate::hooks::{use_translate, Direction, UseTranslate};
use dioxus::prelude::*;

#[component]
pub fn Translate() -> Element {
    let translator = use_translate();

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Перевод".to_string(),
                subtitle: Some("cli: translate (auto / JA→RU / RU→JA)".to_string()),
                actions: None,
            }

            Card { class: Some("grid grid-cols-1 md:grid-cols-3 gap-4".to_string()),
                TranslationInput { translator: translator.clone() }
                TranslationControls { translator: translator.clone() }
                TranslationResult { translator }
            }
        }
    }
}

#[component]
fn TranslationInput(mut translator: UseTranslate) -> Element {
    rsx! {
        Textarea {
            label: Some("ТЕКСТ".to_string()),
            placeholder: Some("日本語 или русский текст".to_string()),
            value: Some(translator.text),
            oninput: Some(EventHandler::new(move |e: Event<FormData>| translator.text.set(e.value()))),
            rows: Some(8),
        }
    }
}

#[component]
fn TranslationControls(mut translator: UseTranslate) -> Element {
    rsx! {
        div { class: "space-y-3",
            DirectionSelector { translator: translator.clone() }
            TranslateButton { translator }
        }
    }
}

#[component]
fn DirectionSelector(mut translator: UseTranslate) -> Element {
    let directions = vec![
        (Direction::Auto, "Авто".to_string()),
        (Direction::JpToRu, "JA → RU".to_string()),
        (Direction::RuToJp, "RU → JA".to_string()),
    ];

    rsx! {
        div { class: "grid grid-cols-3 gap-2",
            for (direction , label) in directions {
                DirectionButton {
                    label,
                    active: (translator.direction)() == direction,
                    onclick: move |_| translator.direction.set(direction),
                }
            }
        }
    }
}

#[component]
fn DirectionButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        Button {
            variant: if active { ButtonVariant::Rainbow } else { ButtonVariant::Outline },
            class: Some("w-full text-sm".to_string()),
            onclick,
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
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
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
