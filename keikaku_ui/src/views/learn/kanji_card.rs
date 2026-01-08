use dioxus::prelude::*;

use crate::components::app_ui::{InfoSection, InfoSectionTone};
use crate::domain::{AnswerActionButtons, FuriganaText, RadicalGrid, Rating, WordCard};
use crate::views::learn::learn_session::{LearnCard, LearnStep};
use keikaku::domain::value_objects::NativeLanguage;

#[component]
pub fn KanjiCardView(
    card: LearnCard,
    current_step: LearnStep,
    show_furigana: bool,
    native_language: NativeLanguage,
    on_show_answer: EventHandler<()>,
    on_rate: EventHandler<Rating>,
) -> Element {
    match current_step {
        LearnStep::Question => rsx! {
            KanjiQuestionView {
                question: card.question,
                show_furigana,
                on_show_answer,
            }
        },
        LearnStep::Answer => rsx! {
            KanjiAnswerView {
                card,
                show_furigana,
                native_language,
                on_rate,
            }
        },
        LearnStep::Completed => rsx! {
            KanjiCompletedView { card, show_furigana, native_language }
        },
    }
}

#[component]
fn KanjiQuestionView(
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
                    super::card_display::QuestionActionButtons { on_show_answer }
                }
            }
        }
    }
}

#[component]
fn KanjiAnswerView(
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
                        KanjiExamplesSection { card: card.clone(), show_furigana }
                    }
                
                }

                // Right column: Action buttons
                div { class: "space-y-2",
                    AnswerActionButtons { on_rate }
                }
            }

            div { class: "space-y-2",
                // Radicals for kanji cards
                if !card.radicals.is_empty() {
                    KanjiRadicalsSection { radicals: card.radicals }
                }
            }
        }
    }
}

#[component]
fn KanjiCompletedView(
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
                KanjiExamplesSection { card: card.clone(), show_furigana }

                // Middle and right columns: Empty for balance
                div { class: "space-y-2" }
                div { class: "space-y-2" }
            }

            // Third row: Radicals on full width
            div { class: "space-y-2",
                // Radicals for kanji cards
                if !card.radicals.is_empty() {
                    KanjiRadicalsSection { radicals: card.radicals }
                }
            }
        }
    }
}

#[component]
fn KanjiExamplesSection(card: LearnCard, show_furigana: bool) -> Element {
    if card.example_words.is_empty() {
        return rsx! {};
    }

    rsx! {
        InfoSection {
            title: "Популярные слова:".to_string(),
            tone: InfoSectionTone::Neutral,
            div { class: "space-y-2",
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
fn KanjiRadicalsSection(radicals: Vec<keikaku::domain::dictionary::RadicalInfo>) -> Element {
    rsx! {
        RadicalGrid { radicals: radicals.clone(), show_kanji_list: true, dense: None }
    }
}
