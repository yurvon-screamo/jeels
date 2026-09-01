use super::{DangerZoneCard, PasswordCard, PersonalDataCard, SettingsCard, legal_card};
use crate::i18n::{native_language_to_locale, t, use_i18n};
use crate::store::AuthStore;
use crate::ui_components::{Card, OfflineBundleCard};
use crate::utils::display_name::{display_name_for, editable_username_for};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::domain::{DailyLoad, NativeLanguage, User};
use origa::use_cases::UpdateUserProfileUseCase;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum AutoSaveStatus {
    #[default]
    Idle,
    Saving,
    Saved,
    Error,
}

const AUTOSAVE_STATUS_DISPLAY_MS: u32 = 1500;

#[component]
pub fn ProfileContent() -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    // Initial value of the editable input: empty for Apple relay profiles
    // that have no meaningful name yet (placeholder question shows instead).
    let stored_username = Memo::new(move |_| {
        auth_store.user.with(|u: &Option<User>| {
            u.as_ref()
                .map(|u| editable_username_for(u.username(), u.email()))
                .unwrap_or_default()
        })
    });

    // Displayed name: email when the username is empty, so the breadcrumbs
    // never render a blank or an opaque relay local part on its own.
    let user_name = Memo::new(move |_| {
        auth_store.user.with(|u: &Option<User>| {
            u.as_ref()
                .map(|u| display_name_for(u.username(), u.email()))
                .unwrap_or_default()
        })
    });

    let native_language = Memo::new(move |_| {
        auth_store.user.with(|u: &Option<User>| {
            u.as_ref()
                .map(|u| *u.native_language())
                .unwrap_or(NativeLanguage::Russian)
        })
    });

    let selected_language = RwSignal::new(native_language.get_untracked());

    Effect::new(move |_| {
        selected_language.set(native_language.get());
    });

    let i18n = use_i18n();
    let i18n_for_sync = i18n;
    Effect::new(move |_| {
        let lang = selected_language.get();
        let locale = native_language_to_locale(&lang);
        tracing::debug!(
            ?locale,
            "Profile Effect: syncing i18n locale from selected_language"
        );
        i18n_for_sync.set_locale(locale);
    });

    let daily_load = Memo::new(move |_| {
        auth_store
            .user
            .with(|u: &Option<User>| u.as_ref().map(|u| *u.daily_load()).unwrap_or_default())
    });

    let selected_daily_load = RwSignal::new(daily_load.get_untracked());

    Effect::new(move |_| {
        selected_daily_load.set(daily_load.get());
    });

    let selected_username = RwSignal::new(stored_username.get_untracked());

    // Re-sync the input after a remote refresh (autosave round-trip). Typing
    // between the save and the refresh may be clobbered — same trade-off the
    // language and daily-load selectors already accept.
    Effect::new(move |_| {
        selected_username.set(stored_username.get());
    });

    let save_status: RwSignal<AutoSaveStatus> = RwSignal::new(AutoSaveStatus::Idle);
    let is_logging_out = RwSignal::new(false);
    let is_deleting = RwSignal::new(false);
    let disposed = StoredValue::new(());
    let is_saving = RwSignal::new(false);
    let needs_resave = RwSignal::new(false);

    let auth_store_save = auth_store.clone();
    let trigger_save = Callback::new(move |_: ()| {
        if is_saving.get() {
            needs_resave.set(true);
            return;
        }

        is_saving.set(true);
        save_status.set(AutoSaveStatus::Saving);

        let repository = auth_store_save.repository().clone();
        let auth_store_clone = auth_store_save.clone();

        spawn_local(async move {
            loop {
                let language = selected_language.get_untracked();
                let daily_load_val = selected_daily_load.get_untracked();
                // NOTE: the name input's editable value is always what gets
                // persisted — including the empty string. For a legacy relay
                // profile (stored garbage name, blank input) a language or
                // daily-load change therefore also normalizes the stored name
                // to empty; the display fallback (email) makes that
                // invisible and it is the intended end state.
                let username_val = selected_username.get_untracked();

                let use_case = UpdateUserProfileUseCase::new(&repository);
                let result = use_case
                    .execute(language, daily_load_val, None, Some(&username_val))
                    .await;

                if disposed.is_disposed() {
                    return;
                }

                if result.is_err() {
                    is_saving.set(false);
                    needs_resave.set(false);
                    save_status.set(AutoSaveStatus::Error);
                    return;
                }

                let _ = auth_store_clone.refresh_user().await;

                if disposed.is_disposed() {
                    return;
                }

                if needs_resave.get_untracked() {
                    needs_resave.set(false);
                    save_status.set(AutoSaveStatus::Saving);
                    continue;
                }

                break;
            }

            is_saving.set(false);
            save_status.set(AutoSaveStatus::Saved);
            gloo_timers::future::TimeoutFuture::new(AUTOSAVE_STATUS_DISPLAY_MS).await;
            if disposed.is_disposed() {
                return;
            }
            save_status.set(AutoSaveStatus::Idle);
        });
    });

    let on_language_change = {
        let trigger = trigger_save;
        Callback::new(move |_lang: NativeLanguage| {
            trigger.run(());
        })
    };

    let on_daily_load_change = {
        let trigger = trigger_save;
        Callback::new(move |_load: DailyLoad| {
            trigger.run(());
        })
    };

    // The Input fires `change` on blur/Enter, so this commits the typed name
    // once — not per keystroke. Skipping the trigger when the value is
    // unchanged avoids a redundant network save on a plain focus pass.
    let on_username_change = {
        let trigger = trigger_save;
        Callback::new(move |(): ()| {
            if selected_username.get_untracked().trim() != stored_username.get_untracked().trim() {
                trigger.run(());
            }
        })
    };

    let on_retry = trigger_save;

    let navigate = use_navigate();
    let navigate_for_logout = navigate.clone();
    let navigate_for_delete = navigate.clone();

    let auth_store_for_logout = auth_store.clone();
    let logout = Callback::new(move |_| {
        let nav = navigate_for_logout.clone();
        let auth_store_clone = auth_store_for_logout.clone();
        let is_logging_out_signal = is_logging_out;

        is_logging_out_signal.set(true);

        spawn_local(async move {
            let _ = auth_store_clone.logout().await;
            nav("/login", Default::default());
        });
    });

    let auth_store_for_delete = auth_store.clone();
    let delete_account = Callback::new(move |_| {
        let nav = navigate_for_delete.clone();
        let auth_store_clone = auth_store_for_delete.clone();

        is_deleting.set(true);

        spawn_local(async move {
            let _ = auth_store_clone.delete_account().await;
            if disposed.is_disposed() {
                return;
            }
            nav("/login", Default::default());
        });
    });

    view! {
        <div class="profile-layout" data-testid="profile-content">
            <div class="profile-breadcrumbs">
                <span class="profile-breadcrumbs-label">{t!(i18n, home.profile)}</span>
                <span class="profile-breadcrumbs-separator">"/"</span>
                <span class="profile-breadcrumbs-current">{move || user_name.get()}</span>
            </div>

            <div class="profile-grid">
                <div class="profile-col">
                    <Card shadow=Signal::derive(|| true)>
                        <PersonalDataCard
                            username={selected_username}
                            selected_language={selected_language}
                            selected_daily_load={selected_daily_load}
                            save_status={Signal::derive(move || save_status.get())}
                            on_username_change={on_username_change}
                            on_language_change={on_language_change}
                            on_daily_load_change={on_daily_load_change}
                            on_retry={on_retry}
                            test_id="profile-personal-data"
                        />
                    </Card>
                    <Card shadow=Signal::derive(|| true)>
                        <OfflineBundleCard test_id="profile-offline-bundle" />
                    </Card>
                </div>

                <div class="profile-col">
                    <Card shadow=Signal::derive(|| true)>
                        <PasswordCard test_id="profile-password" />
                    </Card>
                    <Card>
                        <SettingsCard test_id="profile-settings" />
                        {legal_card(Signal::derive(|| "profile-legal".to_string()))}
                    </Card>
                    <DangerZoneCard
                        on_logout={logout}
                        on_delete_account={delete_account}
                        is_logging_out={Signal::derive(move || is_logging_out.get())}
                        is_deleting={Signal::derive(move || is_deleting.get())}
                        test_id="profile-danger-zone"
                    />
                </div>
            </div>
        </div>
    }
}
