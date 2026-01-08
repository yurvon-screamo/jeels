use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use keikaku::application::UserRepository;
use keikaku::domain::UserSettings;

use crate::components::app_ui::{ErrorCard, SectionHeader};
use crate::views::profile::SettingsForm;
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Profile() -> Element {
    let settings_resource = use_resource(fetch_user_settings);
    let loading = use_signal(|| false);
    let toast = use_toast();

    // Read resources once and store results
    let settings_read = settings_resource.read();

    match settings_read.as_ref() {
        Some(Ok(settings)) => rsx! {
            ProfileContent {
                settings: settings.clone(),
                on_save: move |updated_settings| {
                    let mut loading = loading;
                    let settings_resource = settings_resource;
                    spawn(async move {
                        loading.set(true);
                        match save_user_settings(updated_settings).await {
                            Ok(_) => {
                                toast
                                    .success(
                                        "Настройки сохранены".to_string(),
                                        ToastOptions::new(),
                                    );
                                let mut settings_resource = settings_resource;
                                settings_resource.restart();
                            }
                            Err(e) => {
                                toast
                                    .error(
                                        format!("Ошибка сохранения: {}", e),
                                        ToastOptions::new(),
                                    );
                            }
                        }
                        loading.set(false);
                    });
                },
                loading: loading(),
            }
        },
        Some(Err(err)) => rsx! {
            ErrorCard { message: format!("Ошибка загрузки настроек: {}", err) }
        },
        None => rsx! {
            div { class: "bg-bg min-h-screen text-text-main px-6 py-8", "Загрузка..." }
        },
    }
}

#[component]
fn ProfileContent(
    settings: UserSettings,
    on_save: EventHandler<UserSettings>,
    loading: bool,
) -> Element {
    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Профиль пользователя".to_string(),
                subtitle: Some("Настройки приложения и сервисов".to_string()),
                actions: None,
            }

            SettingsForm { settings, on_save, loading }
        }
    }
}

async fn fetch_user_settings() -> Result<UserSettings, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;

    let user = repo
        .find_by_id(user_id)
        .await
        .map_err(to_error)?
        .ok_or("Пользователь не найден".to_string())?;

    Ok(user.settings().clone())
}

async fn save_user_settings(settings: UserSettings) -> Result<(), String> {
    use keikaku::application::use_cases::update_user_settings::{
        UpdateUserSettingsRequest, UpdateUserSettingsUseCase,
    };

    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;

    let request = UpdateUserSettingsRequest {
        llm: Some(settings.llm().clone()),
        duolingo_jwt_token: Some(settings.duolingo_jwt_token().map(|s| s.to_string())),
    };

    UpdateUserSettingsUseCase::new(repo)
        .execute(user_id, request)
        .await
        .map_err(to_error)?;

    Ok(())
}
