use dioxus::prelude::*;

use crate::domain::{AnswerActionButtons, FuriganaText, QuestionActionButtons, Rating, WordCard};
use crate::ui::{Card, InfoGrid, InfoSection, InfoSectionTone, Pill};
use crate::views::learn::learn_session::{CardType, LearnCard, LearnStep, SimilarCard};

#[component]
pub fn LearnCardDisplay(
    cards: Vec<super::LearnCard>,
    current_index: usize,
    current_step: super::LearnStep,
    show_furigana: bool,
    on_show_answer: EventHandler<()>,
    on_next: EventHandler<()>,
    on_rate: EventHandler<Rating>,
) -> Element {
    let card = cards.get(current_index).cloned();

    if let Some(card) = card {
        rsx! {
            Card { class: Some("space-y-4 transition-all duration-300".to_string()),

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
                        on_rate: move |rating| on_rate.call(rating),
                    }
                } else {
                    CardCompletedView { card: card.clone(), show_furigana }
                }
            }
        }
    } else {
        rsx! {
            Card { class: Some("space-y-4".to_string()),
                crate::ui::Paragraph { class: Some("text-sm text-slate-500 text-center".to_string()),
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
        div { class: "space-y-4",
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
                div { class: "lg:col-span-2 space-y-4",
                    // Question
                    div { class: "space-y-2",
                        div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                            "Вопрос"
                        }
                        WordCard { text: question, show_furigana, class: None }
                    }
                }

                div { class: "space-y-4",
                    QuestionActionButtons { on_show_answer }
                }
            }
        }
    }
}

#[component]
fn CardAnswerView(card: LearnCard, show_furigana: bool, on_rate: EventHandler<Rating>) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Question + Answer
                div { class: "lg:col-span-2 space-y-4",
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
                        div { class: "space-y-4",
                            // Answer
                            div { class: "space-y-2",
                                div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                                    "Ответ"
                                }
                                div { class: "relative",
                                    WordCard {
                                        text: card.answer.clone(),
                                        show_furigana,
                                        class: Some("text-2xl md:text-3xl".to_string()),
                                    }
                                }
                            }
                        }

                        // Middle column: Examples
                        ExamplesSection { card: card.clone(), show_furigana }
                    }

                }

                // Right column: Action buttons
                div { class: "space-y-4",
                    AnswerActionButtons { on_rate }
                }
            }

            div { class: "space-y-4",
                // Kanji info for vocabulary cards
                if matches!(card.card_type, CardType::Vocabulary) && !card.kanji_info.is_empty() {
                    KanjiInfoSection { kanji_info: card.kanji_info, show_furigana }
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
fn CardCompletedView(card: LearnCard, show_furigana: bool) -> Element {
    rsx! {
        div { class: "space-y-6",
            // First row: Answer on left (same width as Question), empty space on right
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Answer
                div { class: "lg:col-span-2 space-y-4",
                    WordCard {
                        text: card.answer.clone(),
                        show_furigana,
                        class: None,
                    }
                }

                // Right column: Empty for consistency
                div { class: "space-y-4" }
            }

            // Second row: Examples on left, empty on right (same layout as Answer+Examples in active view)
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Examples (same position as Answer in active view)
                ExamplesSection { card: card.clone(), show_furigana }

                // Middle and right columns: Empty for balance
                div { class: "space-y-4" }
                div { class: "space-y-4" }
            }

            // Third row: Kanji info on full width
            div { class: "space-y-4",
                // Kanji info for vocabulary cards
                if matches!(card.card_type, CardType::Vocabulary) && !card.kanji_info.is_empty() {
                    KanjiInfoSection { kanji_info: card.kanji_info, show_furigana }
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
                            class: None,
                        }
                        div { class: "text-slate-600 text-sm", "{example.translation()}" }
                    }
                }
                for example in card.example_words.iter() {
                    div { class: "flex flex-col gap-1",
                        FuriganaText {
                            text: example.word().to_string(),
                            show_furigana,
                            class: None,
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
) -> Element {
    rsx! {
        InfoSection {
            title: "Радикалы:".to_string(),
            tone: InfoSectionTone::Blue,
            InfoGrid {
                for kanji in kanji_info.iter() {
                    div { class: "bg-white rounded p-3 space-y-2",
                        div { class: "flex items-center gap-2",
                            span { class: "text-2xl font-bold text-blue-600", "{kanji.kanji()}" }
                            Pill {
                                text: format!("N{}", kanji.jlpt().as_number()),
                                tone: None,
                            }
                        }
                        p { class: "text-sm text-slate-700", "{kanji.description()}" }
                        if !kanji.radicals().is_empty() {
                            div { class: "flex flex-wrap gap-1",
                                for radical in kanji.radicals() {
                                    Pill {
                                        text: format!("{} - {}", radical.radical(), radical.name()),
                                        tone: None,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RadicalsSection(radicals: Vec<keikaku::domain::dictionary::RadicalInfo>) -> Element {
    rsx! {
        InfoSection {
            title: "Радикалы:".to_string(),
            tone: InfoSectionTone::Purple,
            InfoGrid {
                for radical in radicals.iter() {
                    div { class: "bg-white rounded p-3 space-y-2",
                        div { class: "flex items-center gap-2",
                            span { class: "text-xl font-bold text-purple-600", "{radical.radical()}" }
                            span { class: "text-sm text-slate-600", "{radical.stroke_count()} черт" }
                        }
                        p { class: "font-medium text-slate-800", "{radical.name()}" }
                        p { class: "text-sm text-slate-700", "{radical.description()}" }
                    }
                }
            }
        }
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
