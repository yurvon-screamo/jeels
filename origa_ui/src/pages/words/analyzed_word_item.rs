use crate::i18n::{td_string, use_i18n};
use crate::pages::icons::{CHECK_CIRCLE_ICON, ICON_CLASS_KNOWN, ICON_CLASS_NEW, PLUS_CIRCLE_ICON};
use crate::ui_components::{
    Checkbox, FuriganaText, MarkdownText, MarkdownVariant, Tooltip, TooltipPlacementMode,
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

    let (status_icon, icon_class) = if analyzed_word.is_known {
        (CHECK_CIRCLE_ICON, ICON_CLASS_KNOWN)
    } else {
        (PLUS_CIRCLE_ICON, ICON_CLASS_NEW)
    };

    let tooltip_text = move || {
        if analyzed_word.is_known {
            td_string!(i18n.get_locale(), common.tooltip_known).to_string()
        } else {
            td_string!(i18n.get_locale(), common.tooltip_new).to_string()
        }
    };

    view! {
        <div
            class="group flex items-start gap-4 py-3 px-4 border-b border-[var(--border-dark)] hover:bg-[var(--bg-aged)] transition-colors cursor-pointer"
            data-testid="words-drawer-item"
            on:click=move |_| on_toggle.run(())
        >
            <div class="pt-1">
                <Checkbox
                    checked=Signal::derive(move || is_selected.get())
                    on_change=Callback::new(move |_| on_toggle.run(()))
                />
            </div>

            <div class="flex-1 flex flex-col gap-1">
                <div class="flex items-center gap-2">
                    <div class="text-xl font-serif tracking-wide">
                        <FuriganaText
                            text=analyzed_word.base_form.clone()
                            known_kanji=known_kanji.clone()
                        />
                    </div>

                        <Tooltip text=Signal::derive(tooltip_text) placement_mode=TooltipPlacementMode::ForceBottom>
                        <span class=format!("{} opacity-60 group-hover:opacity-100 transition-opacity", icon_class)
                              inner_html=status_icon
                        />
                    </Tooltip>
                </div>

                {move || {
                    let known_kanji = known_kanji.clone();
                    analyzed_word.meaning.clone().map(move |meaning| {
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
