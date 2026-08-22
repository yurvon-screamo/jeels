use crate::i18n::*;
use crate::ui_components::{
    FuriganaText, KanjiViewMode, KanjiWritingSection, MarkdownText, MarkdownVariant, ReadingGroup,
    ReadingItem, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use origa::domain::NativeLanguage;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct RadicalDisplay {
    pub symbol: char,
    pub name: String,
    pub description: String,
}

#[component]
pub fn KanjiCardDetails(
    kanji: String,
    name: String,
    radicals: Option<Vec<RadicalDisplay>>,
    example_words: Option<Vec<(String, String)>>,
    on_readings: Option<Vec<ReadingItem>>,
    kun_readings: Option<Vec<ReadingItem>>,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let i18n = use_i18n();
    let kanji_stored = StoredValue::new(kanji);
    let name_stored = StoredValue::new(name);
    let radicals_stored = StoredValue::new(radicals);
    let examples_stored = StoredValue::new(example_words);
    let on_readings_stored = StoredValue::new(on_readings);
    let kun_readings_stored = StoredValue::new(kun_readings);

    let details_expanded = RwSignal::new(false);
    let more_text =
        Signal::derive(move || i18n.get_keys().common().more_details().inner().to_string());
    let collapse_text =
        Signal::derive(move || i18n.get_keys().common().collapse().inner().to_string());

    view! {
        <div class="my-6 space-y-4 max-w-max mx-auto">
            // Lesson cards inherit the muted-tag rare styling from ReadingGroup,
            // but the "rare" hint label is intentionally omitted: the lesson is
            // an active-recall context where a textual rare-marker would pull
            // attention toward the very readings we are de-emphasising. The
            // muted style alone is enough; the explanatory label lives on the
            // kanji-detail page where the student is browsing, not recalling.
            <ReadingGroup
                label=Signal::derive(move || i18n.get_keys().lesson().on_yomi().inner().to_string())
                readings=on_readings_stored
            />
            <ReadingGroup
                label=Signal::derive(move || i18n.get_keys().lesson().kun_yomi().inner().to_string())
                readings=kun_readings_stored
            />

            <div class="flex gap-4 items-start text-left">
                <div class="w-16 shrink-0">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {t!(i18n, lesson.meaning)}
                    </Text>
                </div>
                <div class="flex px-2 py-1">
                    <Text size=TextSize::Large class="text-primary">
                        {name_stored.get_value()}
                    </Text>
                </div>
            </div>
        </div>

        <div class="mt-3">
            <button
                class="font-mono text-sm text-[var(--fg-muted)] cursor-pointer hover:text-[var(--fg-black)] underline underline-offset-4 decoration-[var(--border-light)]"
                on:click=move |_| details_expanded.update(|v| *v = !*v)
            >
                {move || if details_expanded.get() { collapse_text.get() } else { more_text.get() }}
            </button>
        </div>

        <Show when=move || details_expanded.get()>
            <div class="my-6 space-y-4 max-w-max mx-auto">
                <Show when=move || radicals_stored.get_value().is_some()>
                    <div class="flex gap-4 items-start text-left">
                        <div class="w-16 shrink-0">
                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                {t!(i18n, lesson.radicals)}
                            </Text>
                        </div>
                        <div class="flex flex-wrap gap-2">
                            {move || {
                                radicals_stored
                                    .get_value()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|radical| {
                                        // Empty name/description mean "no
                                        // translation for this language yet"
                                        // (see RadicalInfo::name) — the row
                                        // degrades to the bare symbol instead
                                        // of rendering blank lines. Option
                                        // renders as nothing when None.
                                        let name_view = (!radical.name.is_empty()).then(|| {
                                            view! {
                                                <Text size=TextSize::Small class="text-muted-foreground">
                                                    {radical.name.clone()}
                                                </Text>
                                            }
                                        });
                                        let description_view =
                                            (!radical.description.is_empty()).then(|| {
                                                view! {
                                                    <Text size=TextSize::Small class="text-muted-foreground text-xs">
                                                        {radical.description.clone()}
                                                    </Text>
                                                }
                                            });
                                        view! {
                                            <div class="flex items-center gap-2 px-2 py-1 bg-secondary/30 rounded">
                                                <Text size=TextSize::Large class="text-primary">
                                                    {radical.symbol}
                                                </Text>
                                                <div class="flex flex-col">
                                                    {name_view}
                                                    {description_view}
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </div>
                    </div>
                </Show>

                <div class="flex gap-4 items-start text-left">
                    <div class="w-16 shrink-0">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            {t!(i18n, lesson.writing)}
                        </Text>
                    </div>
                    <KanjiWritingSection
                        kanji=kanji_stored.get_value()
                        mode=KanjiViewMode::Frames
                    />
                </div>
            </div>

            <Show when=move || examples_stored.get_value().is_some()>
                <div class="my-6">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mb-3 block text-left">
                        {t!(i18n, lesson.examples)}
                    </Text>
                    <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 text-left">
                        {move || {
                            examples_stored.get_value().map(|examples| {
                                examples
                                    .into_iter()
                                    .map(|(word, meaning)| {
                                        let meaning_stored = StoredValue::new(meaning);
                                        view! {
                                            <div class="p-2 bg-[var(--bg-aged)] rounded">
                                                <Text size=TextSize::Default class="font-bold">
                                                    <FuriganaText text=word known_kanji=known_kanji.get() native_language=native_language with_kanji_tooltip=true/>
                                                </Text>
                                                <MarkdownText
                                                    content=Signal::derive(move || meaning_stored.get_value())
                                                    variant=MarkdownVariant::Compact
                                                    class="text-[var(--fg-muted)]"
                                                    known_kanji=known_kanji.get()
                                                />
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                        }}
                    </div>
                </div>
            </Show>
        </Show>
    }
}
