use crate::i18n::{td_string, use_i18n};
use crate::pages::icons::{
    CHECK_CIRCLE_ICON, ICON_CLASS_KNOWN, ICON_CLASS_MUTED, ICON_CLASS_NEW, PLUS_CIRCLE_ICON,
    X_CIRCLE_ICON,
};
use crate::ui_components::{Checkbox, FuriganaText, MarkdownText, MarkdownVariant, Tooltip};
use leptos::prelude::*;
use origa::domain::WordImportOutcome;
use std::collections::HashSet;

#[component]
pub fn SetWordItem(
    word: String,
    known_meaning: Option<String>,
    outcome: WordImportOutcome,
    selected_words: RwSignal<HashSet<String>>,
    known_kanji: HashSet<char>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let word_for_memo = word.clone();
    let is_selected = Memo::new(move |_| selected_words.get().contains(&word_for_memo));

    // Words without a dictionary entry cannot be imported — the import
    // would fail them — so they render as permanently unselectable.
    let is_importable = outcome != WordImportOutcome::NoDictionaryEntry;

    let (status_icon, tooltip_text, icon_class) = match outcome {
        WordImportOutcome::New => (
            PLUS_CIRCLE_ICON,
            td_string!(i18n.get_locale(), common.tooltip_new),
            ICON_CLASS_NEW,
        ),
        WordImportOutcome::AlreadyExists => (
            CHECK_CIRCLE_ICON,
            td_string!(i18n.get_locale(), common.tooltip_known),
            ICON_CLASS_KNOWN,
        ),
        WordImportOutcome::DuplicateInSelection => (
            CHECK_CIRCLE_ICON,
            td_string!(i18n.get_locale(), sets.tooltip_duplicate_in_selection),
            ICON_CLASS_KNOWN,
        ),
        WordImportOutcome::NoDictionaryEntry => (
            X_CIRCLE_ICON,
            td_string!(i18n.get_locale(), sets.tooltip_no_dictionary),
            ICON_CLASS_MUTED,
        ),
    };

    view! {
        <div
            class=move || {
                let base = "group flex items-start gap-4 py-3 px-4 border-b border-[var(--border-dark)] transition-colors";
                if is_importable {
                    format!("{base} hover:bg-[var(--bg-aged)] cursor-pointer")
                } else {
                    format!("{base} opacity-60")
                }
            }
            data-testid="sets-drawer-item"
            on:click=move |_| {
                if is_importable {
                    on_toggle.run(());
                }
            }
        >
            <div class="pt-1">
                <Checkbox
                    checked=Signal::derive(move || is_selected.get() && is_importable)
                    on_change=Callback::new(move |_| {
                        if is_importable {
                            on_toggle.run(());
                        }
                    })
                />
            </div>

            <div class="flex-1 flex flex-col gap-1">
                <div class="flex items-center gap-2">
                    <div class="text-xl font-serif tracking-wide">
                        <FuriganaText
                            text=word.clone()
                            known_kanji=known_kanji.clone()
                        />
                    </div>

                    <Tooltip text=Signal::derive(move || tooltip_text.to_string())>
                        <span class=format!("{} opacity-60 group-hover:opacity-100 transition-opacity", icon_class)
                              inner_html=status_icon
                        />
                    </Tooltip>
                </div>

                {move || {
                    let known_kanji = known_kanji.clone();
                    known_meaning.clone().map(move |meaning| {
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
            </div>
        </div>
    }
}
