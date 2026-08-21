use std::collections::HashSet;

use crate::ui_components::{MarkdownText, MarkdownVariant, WordTranslations};
use leptos::prelude::*;
use origa::domain::{Card, CardAnswer, NativeLanguage};
use tracing::warn;

use super::card_type::CardType;

pub struct CardAnswerData {
    pub translations: Option<Vec<String>>,
    pub description: Option<String>,
    pub text: String,
}

pub fn extract_card_answer(
    card: &Card,
    native_language: &NativeLanguage,
    card_type: &CardType,
) -> CardAnswerData {
    match card.answer(native_language) {
        Ok(CardAnswer::Vocabulary {
            translations,
            description,
        }) => CardAnswerData {
            translations: Some(translations),
            description,
            text: String::new(),
        },
        Ok(CardAnswer::Text(s)) => CardAnswerData {
            translations: None,
            description: None,
            text: s,
        },
        Err(e) => {
            warn!(
                card_type = ?card_type,
                content_key = %card.content_key(),
                error = %e,
                "Failed to get card answer"
            );
            CardAnswerData {
                translations: None,
                description: None,
                text: String::new(),
            }
        },
    }
}

#[component]
pub fn CardAnswerDisplay(
    #[prop(into)] translations: Signal<Option<Vec<String>>>,
    #[prop(into)] description: Signal<Option<String>>,
    #[prop(into)] text: Signal<String>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    view! {
        <div class="mt-4 text-center">
            <Show
                when=move || translations.get().is_some()
                fallback=move || {
                    view! {
                        <Show when=move || !text.get().is_empty()>
                            <div class="lesson-answer">
                                <MarkdownText
                                    content=text
                                    variant=Signal::derive(|| MarkdownVariant::Default)
                                    known_kanji=known_kanji.get()
                                />
                            </div>
                        </Show>
                    }
                }
            >
                {move || {
                    let trans = translations.get().unwrap_or_default();
                    let desc = description.get();
                    view! {
                        <div class="lesson-answer">
                            <WordTranslations
                                translations=Signal::derive(move || trans.clone())
                                description=Signal::derive(move || desc.clone())
                            />
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}
