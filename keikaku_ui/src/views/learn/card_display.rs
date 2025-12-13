use dioxus::prelude::*;

use crate::domain::{FuriganaText, Rating, RatingButtons, WordCard};
use crate::ui::{Button, ButtonVariant};
use crate::ui::{Card, Pill, H2};
use crate::views::learn::learn_session::{CardType, LearnCard, LearnStep, SimilarCard};

#[component]
pub fn LearnCardDisplay(
    cards: Vec<super::LearnCard>,
    current_index: usize,
    current_step: super::LearnStep,
    show_furigana: bool,
    similarity_shown: bool,
    on_show_answer: EventHandler<()>,
    on_next: EventHandler<()>,
    on_rate: EventHandler<Rating>,
    on_toggle_similarity: EventHandler<()>,
) -> Element {
    let card = cards.get(current_index).cloned();

    if let Some(card) = card {
        rsx! {
            Card {
                class: Some(
                    format!(
                        "space-y-4 transition-all duration-300 {}",
                        if current_step == super::LearnStep::Question {
                            "border-l-4 border-l-blue-400"
                        } else {
                            "border-l-4 border-l-green-400"
                        },
                    ),
                ),

                // Индикатор состояния
                div { class: "flex items-center gap-2 mb-2",
                    if current_step == super::LearnStep::Question {
                        div { class: "w-2 h-2 bg-blue-400 rounded-full animate-pulse" }
                        span { class: "text-xs text-blue-600 font-medium", "Вопрос" }
                    } else {
                        div { class: "w-2 h-2 bg-green-400 rounded-full" }
                        span { class: "text-xs text-green-600 font-medium", "Ответ" }
                    }
                }

                if current_step == super::LearnStep::Question {
                    QuestionView {
                        question: card.question,
                        show_furigana,
                        on_show_answer: move |_| on_show_answer.call(()),
                    }
                } else if current_step == LearnStep::Answer {
                    CardAnswerView {
                        card: card.clone(),
                        show_furigana,
                        similarity_shown,
                        on_rate: move |rating| on_rate.call(rating),
                        on_toggle_similarity: move |_| on_toggle_similarity.call(()),
                    }
                } else {
                    CardCompletedView {
                        card: card.clone(),
                        show_furigana,
                        similarity_shown,
                    }
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
            WordCard { text: question, show_furigana }
            div { class: "space-y-2",
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
                    onclick: on_show_answer,
                    "Показать ответ (Пробел)"
                }
                div { class: "flex flex-col gap-1 text-xs text-center text-slate-400",
                    p { "Нажмите Пробел, чтобы показать ответ" }
                }
            }
        }
    }
}

#[component]
fn CardAnswerView(
    card: LearnCard,
    show_furigana: bool,
    similarity_shown: bool,
    on_rate: EventHandler<Rating>,
    on_toggle_similarity: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "space-y-6",
            // Main content in two columns
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Question and Answer
                div { class: "lg:col-span-2 space-y-4",
                    // Question
                    div { class: "space-y-2",
                        div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                            "Вопрос"
                        }
                        WordCard { text: card.question.clone(), show_furigana }
                    }

                    // Answer
                    div { class: "space-y-2",
                        div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                            "Ответ"
                        }
                        WordCard { text: card.answer.clone(), show_furigana }
                    }
                }

                // Right column: Examples and additional info
                div { class: "space-y-4",
                    // Examples
                    ExamplesSection { card: card.clone(), show_furigana }

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

            // Controls
            div { class: "flex flex-wrap gap-2 justify-center",
                if card.card_type == CardType::Vocabulary
                    && (!card.similarity.is_empty() || !card.homonyms.is_empty())
                {
                    Button {
                        variant: ButtonVariant::Outline,
                        class: Some("flex-1 min-w-0".to_string()),
                        onclick: on_toggle_similarity,
                        if similarity_shown {
                            "Скрыть связанные"
                        } else {
                            "Показать связанные"
                        }
                    }
                }
            }

            // Related cards panels
            if similarity_shown {
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    if !card.similarity.is_empty() {
                        SimilarityPanel { cards: card.similarity.clone() }
                    }
                    if !card.homonyms.is_empty() {
                        HomonymsPanel { cards: card.homonyms.clone() }
                    }
                }
            }

            // Rating buttons
            RatingButtons { on_rate }
        }
    }
}

#[component]
fn CardCompletedView(card: LearnCard, show_furigana: bool, similarity_shown: bool) -> Element {
    rsx! {
        div { class: "space-y-6",
            // Main content in two columns
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Left column: Answer
                div { class: "lg:col-span-2 space-y-4",
                    WordCard { text: card.answer.clone(), show_furigana }
                }

                // Right column: Examples and additional info
                div { class: "space-y-4",
                    // Examples
                    ExamplesSection { card: card.clone(), show_furigana }

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

            // Related cards panels
            if similarity_shown {
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    if !card.similarity.is_empty() {
                        SimilarityPanel { cards: card.similarity.clone() }
                    }
                    if !card.homonyms.is_empty() {
                        HomonymsPanel { cards: card.homonyms.clone() }
                    }
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
        div { class: "bg-slate-50 rounded-lg p-4 space-y-3",
            H2 { class: Some("text-lg font-semibold text-slate-800".to_string()),
                match card.card_type {
                    CardType::Vocabulary => "Примеры использования:",
                    CardType::Kanji => "Популярные слова:",
                }
            }
            div { class: "space-y-2",
                for example in card.example_phrases.iter() {
                    div { class: "flex flex-col gap-1",
                        FuriganaText {
                            text: example.text().to_string(),
                            show_furigana,
                        }
                        div { class: "text-slate-600 text-sm", "{example.translation()}" }
                    }
                }
                for example in card.example_words.iter() {
                    div { class: "flex flex-col gap-1",
                        FuriganaText {
                            text: example.word().to_string(),
                            show_furigana,
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
        div { class: "bg-blue-50 rounded-lg p-4 space-y-3",
            H2 { class: Some("text-lg font-semibold text-blue-800".to_string()),
                "Информация о кандзи:"
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
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
                                        text: format!("{}: {}", radical.radical(), radical.name()),
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
        div { class: "bg-purple-50 rounded-lg p-4 space-y-3",
            H2 { class: Some("text-lg font-semibold text-purple-800".to_string()),
                "Радикалы:"
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
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
        div { class: "bg-yellow-50 rounded-lg p-4 space-y-3",
            H2 { class: Some("text-lg font-semibold text-yellow-800".to_string()),
                "Связанные карточки:"
            }
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
        div { class: "bg-blue-50 rounded-lg p-4 space-y-3",
            H2 { class: Some("text-lg font-semibold text-blue-800".to_string()), "Омонимы:" }
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
