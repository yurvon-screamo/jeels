use crate::i18n::*;
use crate::pages::login::password_input::PasswordInput;
use crate::store::AuthStore;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn PasswordCard(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    let is_expanded = RwSignal::new(false);
    let current_password = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());
    let is_changing = RwSignal::new(false);
    let error_message = RwSignal::<Option<String>>::new(None);
    let success_message = RwSignal::<Option<String>>::new(None);

    let on_submit = Callback::new(move |_| {
        let client = auth_store.client().clone();
        let old = current_password.get();
        let new_pwd = new_password.get();
        let confirm = confirm_password.get();

        error_message.set(None);
        success_message.set(None);

        if new_pwd.len() < 8 {
            error_message.set(Some(
                td_string!(i18n.get_locale(), profile.password_too_short).to_string(),
            ));
            return;
        }

        if new_pwd != confirm {
            error_message.set(Some(
                td_string!(i18n.get_locale(), profile.password_mismatch).to_string(),
            ));
            return;
        }

        is_changing.set(true);

        spawn_local(async move {
            match client.change_password(&old, &new_pwd, &confirm).await {
                Ok(()) => {
                    success_message.set(Some(
                        td_string!(i18n.get_locale(), profile.password_changed).to_string(),
                    ));
                    current_password.set(String::new());
                    new_password.set(String::new());
                    confirm_password.set(String::new());
                },
                Err(e) => {
                    error_message.set(Some(format!(
                        "{}: {}",
                        td_string!(i18n.get_locale(), profile.password_change_error),
                        e
                    )));
                },
            }
            is_changing.set(false);
        });
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div data-testid=test_id_val class="p-6 space-y-0">
            <div>
                <div
                    class="flex items-center justify-between cursor-pointer select-none touch-manipulation"
                    on:click=move |_| is_expanded.update(|v| *v = !*v)
                >
                    <Text size=TextSize::Large>
                        {t!(i18n, profile.password_title)}
                    </Text>
                    <span class="text-[var(--fg-muted)] text-sm font-mono">
                        {move || if is_expanded.get() { "▲" } else { "▼" }}
                    </span>
                </div>

                <Show when=move || is_expanded.get()>
                    <div class="space-y-6 pt-6">
                        <div class="space-y-4">
                            <PasswordInput
                                value=current_password
                                label=Signal::derive(move || {
                                    td_string!(i18n.get_locale(), profile.current_password).to_string()
                                })
                                autocomplete=Signal::derive(|| "current-password".to_string())
                                test_id=Signal::derive(|| "current-password".to_string())
                            />

                            <PasswordInput
                                value=new_password
                                label=Signal::derive(move || {
                                    td_string!(i18n.get_locale(), profile.new_password).to_string()
                                })
                                autocomplete=Signal::derive(|| "new-password".to_string())
                                test_id=Signal::derive(|| "new-password".to_string())
                            />

                            <PasswordInput
                                value=confirm_password
                                label=Signal::derive(move || {
                                    td_string!(i18n.get_locale(), profile.confirm_password).to_string()
                                })
                                autocomplete=Signal::derive(|| "new-password".to_string())
                                test_id=Signal::derive(|| "confirm-password".to_string())
                            />
                        </div>

                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, profile.password_hint)}
                        </Text>

                        <Show when=move || error_message.get().is_some()>
                            <Alert
                                alert_type=Signal::from(AlertType::Error)
                                message=Signal::derive(move || error_message.get().unwrap_or_default())
                                test_id="password-error"
                            />
                        </Show>

                        <Show when=move || success_message.get().is_some()>
                            <Alert
                                alert_type=Signal::from(AlertType::Success)
                                message=Signal::derive(move || success_message.get().unwrap_or_default())
                                test_id="password-success"
                            />
                        </Show>

                        <Button
                            variant=ButtonVariant::Filled
                            on_click=on_submit
                            disabled=Signal::derive(move || is_changing.get())
                            test_id="change-password-btn"
                        >
                            {move || if is_changing.get() {
                                t!(i18n, profile.changing_password).into_any()
                            } else {
                                t!(i18n, profile.change_password).into_any()
                            }}
                        </Button>
                    </div>
                </Show>
            </div>
        </div>
    }
}
