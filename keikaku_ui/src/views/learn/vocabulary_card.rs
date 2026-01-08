use dioxus::prelude::*;

use crate::components::app_ui::{InfoSection, InfoSectionTone};
use crate::domain::{
    AnswerActionButtons, FuriganaText, KanjiCard as DomainKanjiCard, Rating, WordCard,
};
use crate::views::learn::learn_session::{LearnCard, LearnStep};
use keikaku::domain::value_objects::NativeLanguage;

#[component]
pub fn VocabularyCardView(
    card: LearnCard,
    current_step: LearnStep,
    show_furigana: bool,
    native_language: NativeLanguage,
    on_show_answer: EventHandler<()>,
    on_rate: EventHandler<Rating>,
) -> Element {
    match current_step {
        LearnStep::Question => rsx! {
            VocabularyQuestionView {
                question: card.question,
                show_furigana,
                on_show_answer,
            }
        },
        LearnStep::Answer => rsx! {
            VocabularyAnswerView {
                card,
                show_furigana,
                native_language,
                on_rate,
            }
        },
        LearnStep::Completed => rsx! {
            VocabularyCompletedView { card, show_furigana, native_language }
        },
    }
}

#[component]
fn VocabularyQuestionView(
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
fn VocabularyAnswerView(
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
                        VocabularyExamplesSection { card: card.clone(), show_furigana }
                    }
                
                }

                // Right column: Action buttons
                div { class: "space-y-2",
                    AnswerActionButtons { on_rate }
                }
            }

            div { class: "space-y-2",
                // Kanji info for vocabulary cards
                if !card.kanji_info.is_empty() {
                    VocabularyKanjiInfoSection {
                        kanji_info: card.kanji_info,
                        show_furigana,
                        native_language: native_language.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn VocabularyCompletedView(
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
                VocabularyExamplesSection { card: card.clone(), show_furigana }

                // Middle and right columns: Empty for balance
                div { class: "space-y-2" }
                div { class: "space-y-2" }
            }

            // Third row: Kanji info on full width
            div { class: "space-y-2",
                // Kanji info for vocabulary cards
                if !card.kanji_info.is_empty() {
                    VocabularyKanjiInfoSection {
                        kanji_info: card.kanji_info,
                        show_furigana,
                        native_language: native_language.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn VocabularyExamplesSection(card: LearnCard, show_furigana: bool) -> Element {
    if card.example_phrases.is_empty() {
        return rsx! {};
    }

    rsx! {
        InfoSection {
            title: "Примеры использования:".to_string(),
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
            }
        }
    }
}

#[component]
fn VocabularyKanjiInfoSection(
    kanji_info: Vec<keikaku::domain::dictionary::KanjiInfo>,
    show_furigana: bool,
    native_language: NativeLanguage,
) -> Element {
    rsx! {
        div { class: "space-y-2",
            for kanji in kanji_info.iter() {
                DomainKanjiCard {
                    kanji_info: kanji.clone(),
                    show_furigana,
                    native_language: native_language.clone(),
                    class: Some("border border-slate-200".to_string()),
                }
            }
        }
    }
}
