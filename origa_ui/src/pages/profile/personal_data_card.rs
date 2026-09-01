use crate::i18n::*;
use crate::pages::shared::DailyLoadSelector;
use crate::ui_components::{Input, NativeLanguageToggle};
use leptos::prelude::*;
use origa::domain::{DailyLoad, NativeLanguage};

use super::content::AutoSaveStatus;

#[component]
pub fn PersonalDataCard(
    #[prop(optional, into)] test_id: Signal<String>,
    username: RwSignal<String>,
    selected_language: RwSignal<NativeLanguage>,
    selected_daily_load: RwSignal<DailyLoad>,
    save_status: Signal<AutoSaveStatus>,
    on_username_change: Callback<()>,
    on_language_change: Callback<NativeLanguage>,
    on_daily_load_change: Callback<DailyLoad>,
    on_retry: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let i18n_for_placeholder = i18n;
    let username_placeholder = Signal::derive(move || {
        let locale = i18n_for_placeholder.get_locale();
        td_string!(locale, common.username_placeholder).to_string()
    });
    let on_username_input_change = Callback::new(move |_ev: leptos::ev::Event| {
        on_username_change.run(());
    });

    view! {
        <div data-testid=test_id_val class="p-6">
            <div class="flex flex-col gap-4">
                <div class="profile-section">
                    <label class="label-muted block" for="profile-username-input">
                        {t!(i18n, profile.username)}
                    </label>
                    <Input
                        value=username
                        placeholder=username_placeholder
                        id=Signal::derive(|| "profile-username-input".to_string())
                        name=Signal::derive(|| "display_name".to_string())
                        maxlength=Signal::derive(|| Some(origa::use_cases::USERNAME_MAX_CHARS))
                        class=Signal::derive(|| "w-full".to_string())
                        on_change=on_username_input_change
                        test_id=Signal::derive(|| "profile-username-input".to_string())
                    />
                </div>
                <div class="profile-section-row">
                    <div class="label-muted">{t!(i18n, profile.interface_language)}</div>
                    <NativeLanguageToggle
                        selected_language={selected_language}
                        on_change={on_language_change}
                        test_id=Signal::derive(|| "profile-lang-toggle".to_string())
                    />
                </div>
                <div class="profile-section">
                    <div class="label-muted">{t!(i18n, profile.learning_pace)}</div>
                    <DailyLoadSelector
                        selected_load={selected_daily_load}
                        on_change={on_daily_load_change}
                    />
                </div>

                <div data-testid="profile-autosave-status">
                    {move || match save_status.get() {
                        AutoSaveStatus::Idle => ().into_any(),
                        AutoSaveStatus::Saving => view! {
                            <div class="autosave-status">
                                <span class="autosave-spinner"></span>
                                {t!(i18n, profile.autosave_saving)}
                            </div>
                        }.into_any(),
                        AutoSaveStatus::Saved => view! {
                            <div class="autosave-status autosave-status--saved">
                                {t!(i18n, profile.autosave_saved)}
                            </div>
                        }.into_any(),
                        AutoSaveStatus::Error => view! {
                            <div class="autosave-status autosave-status--error">
                                {t!(i18n, profile.autosave_error)}
                                <button
                                    class="autosave-retry"
                                    on:click=move |_| { on_retry.run(()); }
                                >
                                    {t!(i18n, profile.autosave_retry)}
                                </button>
                            </div>
                        }.into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}
