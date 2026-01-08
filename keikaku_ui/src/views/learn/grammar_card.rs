use dioxus::prelude::*;

use crate::domain::{AnswerActionButtons, Rating, WordCard};
use crate::views::learn::learn_session::{LearnCard, LearnStep};
use keikaku::domain::value_objects::NativeLanguage;

#[component]
pub fn GrammarCardView(
    card: LearnCard,
    current_step: LearnStep,
    show_furigana: bool,
    native_language: NativeLanguage,
    on_show_answer: EventHandler<()>,
    on_rate: EventHandler<Rating>,
) -> Element {
    match current_step {
        LearnStep::Question => rsx! {
            GrammarQuestionView {
                question: card.question,
                show_furigana,
                on_show_answer,
            }
        },
        LearnStep::Answer => rsx! {
            GrammarAnswerView {
                card,
                show_furigana,
                native_language,
                on_rate,
            }
        },
        LearnStep::Completed => rsx! {
            GrammarCompletedView { card, show_furigana, native_language }
        },
    }
}

#[component]
fn GrammarQuestionView(
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
fn GrammarAnswerView(
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

                    // second row: Answer on left, empty on right
                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                        // Left column: Answer
                        div { class: "space-y-2",
                            // Answer
                            div { class: "space-y-2",
                                div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold",
                                    "Ответ"
                                }
                                div { class: "relative",
                                    if let Some(markdown) = &card.markdown_description {
                                        super::card_display::GrammarCardView {
                                            markdown_content: markdown.clone(),
                                            show_furigana,
                                        }
                                    } else {
                                        WordCard {
                                            text: card.answer.clone(),
                                            show_furigana,
                                            class: Some("text-lg md:text-xl".to_string()),
                                        }
                                    }
                                }
                            }
                        }

                        // Middle column: Empty for grammar cards
                        div { class: "space-y-2" }
                    }
                
                }

                // Right column: Action buttons
                div { class: "space-y-2",
                    AnswerActionButtons { on_rate }
                }
            }
        }
    }
}

#[component]
fn GrammarCompletedView(
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
                    if let Some(markdown) = &card.markdown_description {
                        super::card_display::GrammarCardView {
                            markdown_content: markdown.clone(),
                            show_furigana,
                        }
                    } else {
                        WordCard {
                            text: card.answer.clone(),
                            show_furigana,
                            class: None,
                        }
                    }
                }

                // Right column: Empty for consistency
                div { class: "space-y-2" }
            }

            // Second row: Empty for balance
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                div { class: "space-y-2" }
                div { class: "space-y-2" }
                div { class: "space-y-2" }
            }
        }
    }
}
