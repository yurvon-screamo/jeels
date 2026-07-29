use super::grammar_details_expand::GrammarDetailsExpand;
use super::kanji_card_details::{KanjiCardDetails, RadicalDisplay};
use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, Heading, HeadingLevel, MarkdownText, MarkdownVariant,
    Text, TextSize, TranslatorText, TypographyVariant, WordTranslations,
};
use leptos::{ev::MouseEvent, prelude::*};
use origa::domain::{GrammarInfo, NativeLanguage};
use std::collections::HashSet;

#[component]
pub fn LessonCardAnswer(
    question_text: String,
    answer_text: String,
    answer_translations: Option<Vec<String>>,
    answer_description: Option<String>,
    is_expanded: RwSignal<bool>,
    needs_collapse: RwSignal<bool>,
    content_ref: NodeRef<leptos::html::Div>,
    on_toggle: Callback<()>,
    is_kanji: bool,
    is_phrase: bool,
    is_reversed: bool,
    on_readings: Option<Vec<crate::ui_components::ReadingWithFreq>>,
    kun_readings: Option<Vec<crate::ui_components::ReadingWithFreq>>,
    radicals: Option<Vec<RadicalDisplay>>,
    example_words: Option<Vec<(String, String)>>,
    grammar_info: Option<GrammarInfo>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let i18n = use_i18n();
    let question = StoredValue::new(question_text);
    let answer = StoredValue::new(answer_text);
    let answer_translations_stored = StoredValue::new(answer_translations);
    let answer_description_stored = StoredValue::new(answer_description);
    let on_readings_stored = StoredValue::new(on_readings);
    let kun_readings_stored = StoredValue::new(kun_readings);
    let radicals_stored = StoredValue::new(radicals);
    let examples_stored = StoredValue::new(example_words);
    let grammar_info_stored = StoredValue::new(grammar_info);
    // Only meaningful when grammar_info contains a rule_id — controls "More details" expand
    let is_grammar_expanded = RwSignal::new(false);

    view! {
        <div class="text-center">
            <Show
                when=move || is_kanji
                fallback=move || {
                    view! {
                        <Heading level=HeadingLevel::H3 class="mb-2">
                            <Show
                                when=move || is_reversed
                                fallback=move || {
                                    if is_phrase {
                                        view! {
                                            <TranslatorText text=question.get_value() />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <FuriganaText
                                                text=question.get_value()
                                                known_kanji=known_kanji.get()
                                                native_language=native_language
                                                with_kanji_tooltip=true
                                                class=Signal::derive(|| "text-3xl leading-snug".to_string())
                                            />
                                        }.into_any()
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
                    }
                }
            >
                <Heading level=HeadingLevel::H1 class="text-6xl mb-2 text-primary text-center">
                    {question.get_value()}
                </Heading>
            </Show>

            <div
                node_ref=content_ref
                class=move || if is_expanded.get() { "border-t border-[var(--border-light)] pt-4 mt-4" } else { "border-t border-[var(--border-light)] pt-4 mt-4 line-clamp-3" }
            >
                <div class="max-w-max mx-auto space-y-4">
                    <Show
                        when=move || {
                            grammar_info_stored
                                .get_value()
                                .as_ref()
                                .is_some_and(|info| !info.description().is_empty())
                                && !is_kanji
                        }
                    >
                        {move || {
                            grammar_info_stored
                                .get_value()
                                .map(|info| {
                                    view! {
                                        <div class="text-left">
                                            <MarkdownText
                                                content=Signal::stored(info.description().to_string())
                                                variant=Signal::derive(|| MarkdownVariant::Default)
                                                known_kanji=known_kanji.get()
                                            />
                                        </div>
                                    }
                                })
                        }}
                    </Show>
                    <Show
                        when=move || is_kanji
                        fallback=move || {
                            view! {
                                <div class="flex gap-4 items-baseline text-left">
                                    <div class="w-16 shrink-0">
                                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                            {t!(i18n, lesson.answer)}
                                        </Text>
                                    </div>
                                    <Show
                                        when=move || answer_translations_stored.get_value().is_some()
                                        fallback=move || {
                                            view! {
                                                <MarkdownText
                                                    content=Signal::derive(move || answer.get_value())
                                                    variant=Signal::derive(|| MarkdownVariant::Large)
                                                    known_kanji=known_kanji.get()
                                                />
                                            }
                                        }
                                    >
                                        {move || {
                                            let trans = answer_translations_stored.get_value().unwrap_or_default();
                                            let desc = answer_description_stored.get_value();
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
                    >
                        <KanjiCardDetails
                            kanji=question.get_value()
                            name=answer.get_value()
                            radicals=radicals_stored.get_value()
                            example_words=examples_stored.get_value()
                            on_readings=on_readings_stored.get_value()
                            kun_readings=kun_readings_stored.get_value()
                            known_kanji=known_kanji
                            native_language=native_language
                        />
                    </Show>
                </div>
            </div>

            <Show
                when=move || grammar_info_stored
                    .get_value()
                    .as_ref()
                    .is_some_and(|info| info.rule_id().is_some())
                    && !is_kanji
            >
                {move || {
                    grammar_info_stored
                        .get_value()
                        .as_ref()
                        .map(|info| {
                            let rule_id = info.rule_id().unwrap();
                            view! {
                                <GrammarDetailsExpand
                                    rule_id
                                    is_expanded=is_grammar_expanded
                                    known_kanji=known_kanji.get()
                                />
                            }
                        })
                }}
            </Show>

            <Show when=move || needs_collapse.get()>
                <div class="mt-2">
                    <Button
                        variant=ButtonVariant::Ghost
                        on_click=Callback::new(move |_: MouseEvent| on_toggle.run(()))
                    >
                        {move || if is_expanded.get() { t!(i18n, common.collapse).into_any() } else { t!(i18n, common.expand).into_any() }}
                    </Button>
                </div>
            </Show>
        </div>
    }
}
