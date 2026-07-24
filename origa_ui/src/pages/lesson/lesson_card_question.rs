use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, DisplayText, FuriganaText, Heading, HeadingLevel, KanjiViewMode,
    KanjiWritingSection, MarkdownText, MarkdownVariant,
};
use leptos::prelude::*;
use origa::domain::NativeLanguage;
use std::collections::HashSet;

#[component]
pub fn LessonCardQuestion(
    question_text: String,
    kanji: Option<String>,
    is_reversed: bool,
    on_show_answer: Callback<()>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let i18n = use_i18n();
    let question = StoredValue::new(question_text);
    let kanji_stored = StoredValue::new(kanji);

    view! {
        <div class="text-center">
            <Show when=move || kanji_stored.get_value().is_none()>
                <div class="mb-2 sm:mb-4 lesson-question">
                    <Heading level=HeadingLevel::H2>
                        <Show
                            when=move || is_reversed
                            fallback=move || {
                                view! {
                                    <FuriganaText
                                        text=question.get_value()
                                        known_kanji=known_kanji.get()
                                        native_language=native_language
                                        with_kanji_tooltip=true
                                    />
                                }
                            }
                        >
                            <MarkdownText
                                content=Signal::derive(move || question.get_value())
                                variant=Signal::derive(|| MarkdownVariant::Large)
                                known_kanji=known_kanji.get()
                            />
                        </Show>
                    </Heading>
                </div>
            </Show>

            <Show when=move || kanji_stored.get_value().is_some()>
                {move || {
                    kanji_stored.get_value().map(|k| {
                        let k_clone = k.clone();
                        view! {
                            <div class="mb-3 sm:mb-6">
                                <DisplayText>
                                    {k_clone}
                                </DisplayText>
                            </div>
                            <KanjiWritingSection kanji=k mode=KanjiViewMode::Animation />
                        }
                    })
                }}
            </Show>

            <Button
                variant=Signal::derive(|| ButtonVariant::Filled)
                test_id=Signal::derive(|| "lesson-show-answer-btn".to_string())
                on_click=Callback::new(move |_| on_show_answer.run(()))
            >
                {t!(i18n, lesson.show_answer)} <span class="kbd-hint">{t!(i18n, lesson.space_key)}</span>
            </Button>
        </div>
    }
}
