use crate::i18n::{t, td_string, use_i18n};
use crate::pages::icons::{CHECK_CIRCLE_ICON, ICON_CLASS_KNOWN, ICON_CLASS_NEW, PLUS_CIRCLE_ICON};
use crate::ui_components::{
    Checkbox, FuriganaText, MarkdownText, MarkdownVariant, Text, TextSize, Tooltip,
    TooltipPlacementMode, TypographyVariant,
};
use leptos::prelude::*;
use origa::use_cases::AnalyzedWord;
use std::collections::HashSet;

#[component]
pub fn AnalyzedWordItem(
    analyzed_word: AnalyzedWord,
    selected_words: RwSignal<HashSet<String>>,
    known_kanji: HashSet<char>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let base_form = analyzed_word.base_form.clone();
    let is_selected = Memo::new(move |_| selected_words.get().contains(&base_form));

    // Three states: known (already in deck), no-translation (meaning is None
    // — word cannot be added because it's not in our dictionaries), new
    // (available to add).
    let has_meaning = analyzed_word.meaning.is_some();

    let is_known = analyzed_word.is_known;

    let (status_icon, icon_class, tooltip_text) = if is_known {
        (
            CHECK_CIRCLE_ICON,
            ICON_CLASS_KNOWN,
            td_string!(i18n.get_locale(), common.tooltip_known).to_string(),
        )
    } else if !has_meaning {
        (
            "",
            "",
            td_string!(i18n.get_locale(), words.tooltip_no_translation).to_string(),
        )
    } else {
        (
            PLUS_CIRCLE_ICON,
            ICON_CLASS_NEW,
            td_string!(i18n.get_locale(), common.tooltip_new).to_string(),
        )
    };
    let tooltip_stored = StoredValue::new(tooltip_text);
    let status_icon_stored = StoredValue::new(status_icon);
    let icon_class_stored = StoredValue::new(icon_class);

    let is_disabled = analyzed_word.is_known || !has_meaning;
    let meaning_stored = StoredValue::new(analyzed_word.meaning.clone());
    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <div
            class=move || {
                let base = "group flex items-start gap-4 py-3 px-4 border-b border-[var(--border-dark)] transition-colors";
                if is_disabled {
                    format!("{} opacity-40 cursor-not-allowed", base)
                } else {
                    format!("{} hover:bg-[var(--bg-aged)] cursor-pointer", base)
                }
            }
            data-testid="words-drawer-item"
            on:click=move |_| {
                if !is_disabled {
                    on_toggle.run(());
                }
            }
        >
            <div class="pt-1">
                <Checkbox
                    checked=Signal::derive(move || is_selected.get())
                    on_change=Callback::new(move |_| {
                        if !is_disabled {
                            on_toggle.run(());
                        }
                    })
                />
            </div>

            <div class="flex-1 flex flex-col gap-1">
                <div class="flex items-center gap-2">
                    <div class="text-xl font-serif tracking-wide">
                        <FuriganaText
                            text=analyzed_word.base_form.clone()
                            known_kanji=known_kanji_stored.get_value()
                        />
                    </div>

                    <Show when=move || !status_icon_stored.get_value().is_empty() && !is_known>
                        <Tooltip text=Signal::derive(move || tooltip_stored.get_value()) placement_mode=TooltipPlacementMode::ForceBottom>
                            <span class=format!("{} opacity-60 group-hover:opacity-100 transition-opacity", icon_class_stored.get_value())
                                  inner_html=status_icon_stored.get_value()
                            />
                        </Tooltip>
                    </Show>

                    <Show when=move || is_known>
                        <span class=format!("{} opacity-60 group-hover:opacity-100 transition-opacity", icon_class_stored.get_value())
                              inner_html=status_icon_stored.get_value()
                        />
                        <span class="text-[var(--text-label-sm)] text-[var(--accent-sage)] uppercase tracking-[0.1em] font-mono">
                            {t!(i18n, words.already_added)}
                        </span>
                    </Show>
                </div>

                <Show when=move || has_meaning>
                    {move || {
                        let known_kanji = known_kanji_stored.get_value();
                        meaning_stored.get_value().map(move |meaning| {
                            view! {
                                <div class="max-w-md">
                                    <MarkdownText
                                        content=Signal::derive(move || meaning.clone())
                                        known_kanji=known_kanji
                                        variant=MarkdownVariant::Compact
                                        class="text-[var(--fg-muted)]"
                                    />
                                </div>
                            }
                        })
                    }}
                </Show>

                <Show when=move || !has_meaning>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, words.no_translation_found)}
                    </Text>
                </Show>
            </div>
        </div>
    }
}
