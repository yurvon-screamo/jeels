use std::collections::HashSet;

use crate::ui_components::{
    FuriganaText, MarkdownText, ReadingGroup, ReadingItem, Tag, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;

#[component]
pub(in crate::pages::kanji) fn KanjiDetailHeroCard(
    kanji_stored: StoredValue<String>,
    answer_text: Memo<String>,
    on_readings: Vec<ReadingItem>,
    kun_readings: Vec<ReadingItem>,
    has_radicals: bool,
    radicals_stored: StoredValue<String>,
    #[prop(into)] tag_variant: Signal<crate::ui_components::TagVariant>,
    #[prop(into)] tag_label: Signal<String>,
    #[prop(into)] radicals_title: Signal<String>,
    #[prop(into)] on_label: Signal<String>,
    #[prop(into)] kun_label: Signal<String>,
    #[prop(into)] rare_hint: Signal<String>,
) -> impl IntoView {
    let on_readings_stored = StoredValue::new(Some(on_readings));
    let kun_readings_stored = StoredValue::new(Some(kun_readings));

    view! {
        <div class="kanji-detail-hero-card">
            <div class="kanji-detail-hero-header">
                <div class="kanji-detail-hero-kanji">{kanji_stored.get_value()}</div>
                <div class="kanji-detail-hero-info">
                    <div class="kanji-detail-hero-meaning">{answer_text}</div>
                    <div class="kanji-detail-hero-readings">
                        <ReadingGroup
                            label=on_label
                            readings=on_readings_stored
                            rare_hint=rare_hint
                            test_id=Signal::derive(|| "kanji-detail-on-readings".to_string())
                        />
                        <ReadingGroup
                            label=kun_label
                            readings=kun_readings_stored
                            rare_hint=rare_hint
                            test_id=Signal::derive(|| "kanji-detail-kun-readings".to_string())
                        />
                    </div>
                </div>
                <div class="kanji-detail-hero-badge">
                    <Tag variant=tag_variant>{tag_label}</Tag>
                </div>
            </div>

            <Show when=move || has_radicals>
                <div style="margin-top:12px">
                    <span
                        style="font-family:var(--font-mono);font-size:var(--text-2xs,11px);\
                               text-transform:uppercase;letter-spacing:0.1em;\
                               color:var(--fg-muted);margin-right:8px"
                    >
                        {radicals_title}
                    </span>
                    <span
                        style="font-family:var(--font-serif);font-size:var(--text-sm,16px);\
                               color:var(--fg-black)"
                    >
                        {radicals_stored.get_value()}
                    </span>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub(in crate::pages::kanji) fn MobileOverview(
    description: Memo<String>,
    has_radicals: bool,
    radicals_stored: StoredValue<String>,
    #[prop(into)] radicals_title: Signal<String>,
    has_examples: Memo<bool>,
    #[prop(into)] vocabulary_title: Signal<String>,
    example_words: Memo<Vec<(String, String)>>,
    known_kanji: HashSet<char>,
) -> impl IntoView {
    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <Show when=move || !description.get().is_empty()>
            <div class="kanji-detail-section">
                <MarkdownText
                    content=Signal::derive(move || description.get())
                    known_kanji=known_kanji_stored.get_value()
                />
            </div>
        </Show>

        <Show when=move || has_radicals>
            <div class="kanji-detail-section">
                <div class="kanji-detail-section-title">{radicals_title}</div>
                <Text size=TextSize::Default variant=TypographyVariant::Primary>
                    {radicals_stored.get_value()}
                </Text>
            </div>
        </Show>

        <Show when=move || has_examples.get()>
            <div class="kanji-detail-section">
                <div class="kanji-detail-section-title">{vocabulary_title}</div>
                <div class="kanji-vocab-list">
                    <For
                        each=move || example_words.get()
                        key=|(word, _)| word.clone()
                        children=move |(word, meaning): (String, String)| {
                            view! {
                                <div class="kanji-vocab-item">
                                    <div class="kanji-vocab-item-kanji">
                                        {word.chars().next().unwrap_or('?').to_string()}
                                    </div>
                                    <div>
                                        <div class="kanji-vocab-item-reading">
                                            <FuriganaText text=word.clone() known_kanji=known_kanji_stored.get_value()/>
                                        </div>
                                        <div class="kanji-vocab-item-meaning">
                                            <MarkdownText content=Signal::derive(move || meaning.clone()) known_kanji=known_kanji_stored.get_value()/>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </Show>
    }
}
