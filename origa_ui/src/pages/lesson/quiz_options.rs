use crate::i18n::*;
use crate::ui_components::{MarkdownText, MarkdownVariant, Text, TextSize};
use leptos::prelude::*;
use origa::domain::QuizOption;
use std::collections::HashSet;

use super::quiz_result::QuizResult;

#[component]
pub fn QuizOptions(
    options: Vec<QuizOption>,
    selected_option: Option<usize>,
    show_result: Signal<bool>,
    quiz_result: QuizResult,
    on_select_option: Callback<usize>,
    on_dont_know: Callback<()>,
    dont_know_selected: Signal<bool>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="grid grid-cols-2 gap-2 sm:gap-3">
            {move || {
                options
                    .iter()
                    .enumerate()
                    .map(|(index, option): (usize, &QuizOption)| {
                        let is_correct = option.is_correct();
                        let is_selected = selected_option == Some(index);
                        let base_class = "p-2 sm:p-4 border text-left transition-all cursor-pointer relative flex flex-col justify-center min-h-[4rem]";
                        let disabled_class = if show_result.get() { "pointer-events-none" } else { "" };
                        let result_class = quiz_result.option_class(is_correct, is_selected);
                        let selected_ring = if is_selected && !show_result.get() {
                            "ring-2 ring-[var(--accent-olive)]"
                        } else {
                            ""
                        };

                        let shake_class = if show_result.get() && is_selected && matches!(quiz_result, QuizResult::Incorrect) {
                            "anima-shake"
                        } else {
                            ""
                        };

                        let class = format!("{} {} {} {} {}", base_class, disabled_class, result_class, selected_ring, shake_class);
                        let key_hint = format!("[{}]", index + 1);
                        let option_text = option.text().to_string();

                        view! {
                            <button
                                class=class
                                data-testid=format!("quiz-option-{}", index)
                                on:click=move |_| {
                                    if !show_result.get() {
                                        on_select_option.run(index);
                                    }
                                }
                            >
                                <Text size=TextSize::Default>
                                    <MarkdownText
                                        content=Signal::derive(move || option_text.clone())
                                        variant=MarkdownVariant::Default
                                        known_kanji=known_kanji.get()
                                        glue_punctuation=true
                                    />
                                </Text>
                                <Show when=move || !show_result.get()>
                                    <span class="kbd-hint absolute top-1 right-2 text-[var(--fg-muted)] text-xs font-mono opacity-50 pointer-events-none">
                                        {key_hint.clone()}
                                    </span>
                                </Show>
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
        <button
            data-testid="quiz-dont-know-btn"
            class=move || {
                let base = "w-full mt-2 p-2 sm:p-4 border text-center transition-all cursor-pointer flex items-center justify-center gap-2";
                if dont_know_selected.get() {
                    format!("{} quiz-option-neutral ring-2 ring-[var(--accent-olive)]", base)
                } else if show_result.get() {
                    format!("{} quiz-option-dimmed pointer-events-none", base)
                } else {
                    format!("{} quiz-option-neutral", base)
                }
            }
            on:click=move |_| {
                if !show_result.get() {
                    on_dont_know.run(());
                }
            }
        >
            <Text size=TextSize::Default>{t!(i18n, lesson.dont_know)}</Text>
            <Show when=move || !show_result.get()>
                <span class="kbd-hint text-[var(--fg-muted)] text-xs font-mono">{t!(i18n, lesson.space_key)}</span>
            </Show>
        </button>
    }
}
