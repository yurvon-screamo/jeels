use dioxus::prelude::*;

use crate::components::app_ui::{Card, InfoSection, InfoSectionTone, Paragraph};
use crate::domain::{
    AnswerActionButtons, FuriganaText, KanjiCard, QuestionActionButtons, RadicalGrid, Rating,
    WordCard,
};
use crate::views::learn::learn_session::{CardType, LearnCard, LearnStep, SimilarCard};
use keikaku::domain::value_objects::NativeLanguage;

#[component]
pub fn LearnCardDisplay(
    card: Option<super::LearnCard>,
    current_step: super::LearnStep,
    show_furigana: bool,
    native_language: NativeLanguage,
    on_show_answer: EventHandler<()>,
    on_next: EventHandler<()>,
    on_rate: EventHandler<Rating>,
) -> Element {
    if let Some(card) = card {
        rsx! {
            Card { class: Some("space-y-2 transition-all duration-300".to_string()),

                if current_step == super::LearnStep::Question {
                    CardQuestionView {
                        question: card.question,
                        show_furigana,
                        on_show_answer: move |_| on_show_answer.call(()),
                    }
                } else if current_step == LearnStep::Answer {
                    CardAnswerView {
                        card: card.clone(),
                        show_furigana,
                        native_language: native_language.clone(),
                        on_rate: move |rating| on_rate.call(rating),
                    }
                } else {
                    CardCompletedView {
                        card: card.clone(),
                        show_furigana,
                        native_language: native_language.clone(),
                    }
                }
            }
        }
    } else {
        rsx! {
            Card { class: Some("space-y-2".to_string()),
                Paragraph { class: Some("text-sm text-slate-500 text-center".to_string()),
                    "Нет карточек для отображения"
                }
            }
        }
    }
}

#[component]
pub fn QuestionView(
    question: String,
    show_furigana: bool,
    on_show_answer: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "space-y-2",
            WordCard { text: question, show_furigana, class: None }
        }
    }
}

#[component]
fn CardQuestionView(
    question: String,
    show_furigana: bool,
    on_show_answer: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "space-y-6",
            // Main content: Question on left, buttons on right
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Question
                div { class: "lg:col-span-2 space-y-2",
                    // Question
                    div { class: "space-y-2",
                        div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                            "Вопрос"
                        }
                        WordCard { text: question, show_furigana, class: None }
                    }
                }

                div { class: "space-y-2 flex flex-col h-full",
                    QuestionActionButtons { on_show_answer }
                }
            }
        }
    }
}

#[component]
fn CardAnswerView(
    card: LearnCard,
    show_furigana: bool,
    native_language: NativeLanguage,
    on_rate: EventHandler<Rating>,
) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Question + Answer
                div { class: "lg:col-span-2 space-y-2",
                    // first row: Question to full width
                    div { class: "space-y-2",
                        div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                            "Вопрос"
                        }
                        WordCard {
                            text: card.question.clone(),
                            show_furigana,
                            class: None,
                        }
                    }

                    // second row: Answer on left, Examples on right (together same width as Question)
                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                        // Left column: Answer
                        div { class: "space-y-2",
                            // Answer
                            div { class: "space-y-2",
                                div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                                    "Ответ"
                                }
                                div { class: "relative",
                                    WordCard {
                                        text: card.answer.clone(),
                                        show_furigana,
                                        class: Some("text-lg md:text-xl".to_string()),
                                    }
                                }
                            }
                        }

                        // Middle column: Examples
                        ExamplesSection { card: card.clone(), show_furigana }
                    }
                
                }

                // Right column: Action buttons
                div { class: "space-y-2",
                    AnswerActionButtons { on_rate }
                }
            }

            div { class: "space-y-2",
                // Kanji info for vocabulary cards
                if matches!(card.card_type, CardType::Vocabulary) && !card.kanji_info.is_empty() {
                    KanjiInfoSection {
                        kanji_info: card.kanji_info,
                        show_furigana,
                        native_language: native_language.clone(),
                    }
                }

                // Radicals for kanji cards
                if matches!(card.card_type, CardType::Kanji) && !card.radicals.is_empty() {
                    RadicalsSection { radicals: card.radicals }
                }
            }
        }
    }
}

#[component]
fn CardCompletedView(
    card: LearnCard,
    show_furigana: bool,
    native_language: NativeLanguage,
) -> Element {
    rsx! {
        div { class: "space-y-6",
            // First row: Answer on left (same width as Question), empty space on right
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Answer
                div { class: "lg:col-span-2 space-y-2",
                    WordCard {
                        text: card.answer.clone(),
                        show_furigana,
                        class: None,
                    }
                }

                // Right column: Empty for consistency
                div { class: "space-y-2" }
            }

            // Second row: Examples on left, empty on right (same layout as Answer+Examples in active view)
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Examples (same position as Answer in active view)
                ExamplesSection { card: card.clone(), show_furigana }

                // Middle and right columns: Empty for balance
                div { class: "space-y-2" }
                div { class: "space-y-2" }
            }

            // Third row: Kanji info on full width
            div { class: "space-y-2",
                // Kanji info for vocabulary cards
                if matches!(card.card_type, CardType::Vocabulary) && !card.kanji_info.is_empty() {
                    KanjiInfoSection {
                        kanji_info: card.kanji_info,
                        show_furigana,
                        native_language: native_language.clone(),
                    }
                }

                // Radicals for kanji cards
                if matches!(card.card_type, CardType::Kanji) && !card.radicals.is_empty() {
                    RadicalsSection { radicals: card.radicals }
                }
            }
        }
    }
}

#[component]
fn ExamplesSection(card: LearnCard, show_furigana: bool) -> Element {
    if card.example_phrases.is_empty() && card.example_words.is_empty() {
        return rsx! {};
    }

    rsx! {
        InfoSection {
            title: match card.card_type {
                CardType::Vocabulary => "Примеры использования:".to_string(),
                CardType::Kanji => "Популярные слова:".to_string(),
            },
            tone: InfoSectionTone::Neutral,
            div { class: "space-y-2",
                for example in card.example_phrases.iter() {
                    div { class: "flex flex-col gap-1",
                        FuriganaText {
                            text: example.text().to_string(),
                            show_furigana,
                            class: Some("text-lg".to_string()),
                        }
                        div { class: "text-slate-600 text-sm", "{example.translation()}" }
                    }
                }
                for example in card.example_words.iter() {
                    div { class: "flex flex-col gap-1",
                        FuriganaText {
                            text: example.word().to_string(),
                            show_furigana,
                            class: Some("text-lg".to_string()),
                        }
                        div { class: "text-slate-600 text-sm", "{example.meaning()}" }
                    }
                }
            }
        }
    }
}

#[component]
fn KanjiInfoSection(
    kanji_info: Vec<keikaku::domain::dictionary::KanjiInfo>,
    show_furigana: bool,
    native_language: NativeLanguage,
) -> Element {
    rsx! {
        div { class: "space-y-2",
            for kanji in kanji_info.iter() {
                KanjiCard {
                    kanji_info: kanji.clone(),
                    show_furigana,
                    native_language: native_language.clone(),
                    class: Some("border border-slate-200".to_string()),
                }
            }
        }
    }
}

#[component]
fn RadicalsSection(radicals: Vec<keikaku::domain::dictionary::RadicalInfo>) -> Element {
    rsx! {
        RadicalGrid { radicals: radicals.clone(), show_kanji_list: true, dense: None }
    }
}

#[component]
fn SimilarityPanel(cards: Vec<SimilarCard>) -> Element {
    rsx! {
        InfoSection {
            title: "Связанные карточки:".to_string(),
            tone: InfoSectionTone::Yellow,
            div { class: "space-y-2",
                for card in cards.iter() {
                    div { class: "flex flex-col gap-1 p-2 bg-white rounded",
                        div { class: "text-cyan-700 font-medium", "{card.word}" }
                        div { class: "text-slate-600 text-sm", "{card.meaning}" }
                    }
                }
            }
        }
    }
}

#[component]
fn HomonymsPanel(cards: Vec<SimilarCard>) -> Element {
    rsx! {
        InfoSection { title: "Омонимы:".to_string(), tone: InfoSectionTone::Blue,
            div { class: "space-y-2",
                for card in cards.iter() {
                    div { class: "flex flex-col gap-1 p-2 bg-white rounded",
                        div { class: "text-cyan-700 font-medium", "{card.word}" }
                        div { class: "text-slate-600 text-sm", "{card.meaning}" }
                    }
                }
            }
        }
    }
}
