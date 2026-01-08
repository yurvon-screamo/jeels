use dioxus::prelude::*;

use crate::components::app_ui::{Card, Paragraph};
use crate::domain::Rating;
use crate::views::learn::learn_session::CardType;
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

                match card.card_type {
                    CardType::Vocabulary => rsx! {
                        super::vocabulary_card::VocabularyCardView {
                            card: card.clone(),
                            current_step,
                            show_furigana,
                            native_language: native_language.clone(),
                            on_show_answer: move |_| on_show_answer.call(()),
                            on_rate: move |rating| on_rate.call(rating),
                        }
                    },
                    CardType::Kanji => rsx! {
                        super::kanji_card::KanjiCardView {
                            card: card.clone(),
                            current_step,
                            show_furigana,
                            native_language: native_language.clone(),
                            on_show_answer: move |_| on_show_answer.call(()),
                            on_rate: move |rating| on_rate.call(rating),
                        }
                    },
                    CardType::Grammar => rsx! {
                        super::grammar_card::GrammarCardView {
                            card: card.clone(),
                            current_step,
                            show_furigana,
                            native_language: native_language.clone(),
                            on_show_answer: move |_| on_show_answer.call(()),
                            on_rate: move |rating| on_rate.call(rating),
                        }
                    },
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
pub fn QuestionActionButtons(on_show_answer: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex flex-col justify-center h-full",
            button {
                class: "bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-6 rounded-lg transition-colors duration-200 shadow-sm",
                onclick: move |_| on_show_answer.call(()),
                "Показать ответ"
            }
        }
    }
}

#[component]
pub fn GrammarCardView(markdown_content: String, show_furigana: bool) -> Element {
    // Simple markdown renderer for grammar descriptions
    fn render_markdown(text: &str, show_furigana: bool) -> Vec<Element> {
        let mut elements = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("# ") {
                // Header 1
                elements.push(rsx! {
                    h1 { class: "text-xl font-bold mb-3 text-slate-800",
                        {line.strip_prefix("# ").unwrap_or(line)}
                    }
                });
            } else if line.starts_with("## ") {
                // Header 2
                elements.push(rsx! {
                    h2 { class: "text-lg font-semibold mb-2 text-slate-700",
                        {line.strip_prefix("## ").unwrap_or(line)}
                    }
                });
            } else if line.starts_with("### ") {
                // Header 3
                elements.push(rsx! {
                    h3 { class: "text-base font-medium mb-2 text-slate-600",
                        {line.strip_prefix("### ").unwrap_or(line)}
                    }
                });
            } else if line.starts_with("* ") || line.starts_with("- ") {
                // List item
                let content = if line.starts_with("* ") {
                    line.strip_prefix("* ").unwrap_or(line)
                } else {
                    line.strip_prefix("- ").unwrap_or(line)
                };

                elements.push(rsx! {
                    li { class: "ml-4 mb-1", {render_inline_markdown(content, show_furigana)} }
                });
            } else {
                // Regular paragraph
                elements.push(rsx! {
                    p { class: "mb-2 text-slate-600 leading-relaxed",
                        {render_inline_markdown(line, show_furigana)}
                    }
                });
            }
        }

        elements
    }

    fn render_inline_markdown(text: &str, _show_furigana: bool) -> Element {
        // Simple inline markdown rendering
        let mut result = text.to_string();

        // Bold: **text**
        result = result.replace("**", "<strong>").replacen(
            "<strong>",
            "<strong class=\"font-semibold\">",
            1,
        );
        result = result.replace("**", "</strong>");

        // Italic: *text*
        result = result
            .replace("*", "<em>")
            .replacen("<em>", "<em class=\"italic\">", 1);
        result = result.replace("*", "</em>");

        // Code: `text`
        result = result.replace("`", "<code>").replacen(
            "<code>",
            "<code class=\"bg-slate-100 px-1 py-0.5 rounded text-sm font-mono\">",
            1,
        );
        result = result.replace("`", "</code>");

        // Render as HTML (unsafe but controlled)
        rsx! {
            div {
                class: "prose prose-sm max-w-none",
                dangerous_inner_html: "{result}",
            }
        }
    }

    rsx! {
        div { class: "space-y-4",
            for element in render_markdown(&markdown_content, show_furigana) {
                {element}
            }
        }
    }
}
