use crate::i18n::{td_string, use_i18n};
use leptos::prelude::*;
use origa::domain::NativeLanguage;

#[component]
pub fn NativeLanguageToggle(
    selected_language: RwSignal<NativeLanguage>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] on_change: Option<Callback<NativeLanguage>>,
) -> impl IntoView {
    let i18n = use_i18n();

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let en_class = Signal::derive(move || {
        if selected_language.get() == NativeLanguage::English {
            "text-[var(--fg-black)] border-b border-[var(--fg-black)] cursor-default"
        } else {
            "text-[var(--fg-muted)] border-b border-transparent hover:text-[var(--fg-black)] hover:border-[var(--border-light)] cursor-pointer"
        }
    });

    let ru_class = Signal::derive(move || {
        if selected_language.get() == NativeLanguage::Russian {
            "text-[var(--fg-black)] border-b border-[var(--fg-black)] cursor-default"
        } else {
            "text-[var(--fg-muted)] border-b border-transparent hover:text-[var(--fg-black)] hover:border-[var(--border-light)] cursor-pointer"
        }
    });

    view! {
        <div
            class="inline-flex items-center gap-2 font-mono text-[11px] uppercase tracking-[0.15em]"
            role="group"
            aria-label=move || td_string!(i18n.get_locale(), common.language_aria_label)
            data-testid=test_id_val
        >
            <button
                type="button"
                data-testid="lang-toggle-en"
                class=move || format!(
                    "bg-transparent p-0 transition-colors duration-150 ease-in-out anima-focus-ring {}",
                    en_class.get()
                )
                aria-current=move || if selected_language.get() == NativeLanguage::English { "true" } else { "false" }
                on:click=move |_| {
                    selected_language.set(NativeLanguage::English);
                    if let Some(cb) = &on_change { cb.run(NativeLanguage::English); }
                }
            >
                "EN"
            </button>

            <span class="text-[var(--border-light)] select-none pointer-events-none">"|"</span>

            <button
                type="button"
                data-testid="lang-toggle-ru"
                class=move || format!(
                    "bg-transparent p-0 transition-colors duration-150 ease-in-out anima-focus-ring {}",
                    ru_class.get()
                )
                aria-current=move || if selected_language.get() == NativeLanguage::Russian { "true" } else { "false" }
                on:click=move |_| {
                    selected_language.set(NativeLanguage::Russian);
                    if let Some(cb) = &on_change { cb.run(NativeLanguage::Russian); }
                }
            >
                "RU"
            </button>
        </div>
    }
}
