use dioxus::prelude::*;
use keikaku::application::UserRepository;
use keikaku::domain::UserSettings;

use crate::ui::{Card, ErrorCard, Paragraph, SectionHeader};
use crate::views::profile::SettingsForm;
use crate::{ensure_user, to_error, DEFAULT_USERNAME};
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Profile() -> Element {
    let settings_resource = use_resource(fetch_user_settings);
    let notification = use_signal(|| Notification::None);
    let loading = use_signal(|| false);

    // Read resources once and store results
    let settings_read = settings_resource.read();

    match settings_read.as_ref() {
        Some(Ok(settings)) => rsx! {
            ProfileContent {
                settings: settings.clone(),
                on_save: move |updated_settings| {
                    let mut notification = notification;
                    let mut loading = loading;
                    let settings_resource = settings_resource;
                    spawn(async move {
                        loading.set(true);
                        match save_user_settings(updated_settings).await {
                            Ok(_) => {
                                notification
                                    .set(
                                        Notification::Success(
                                            "Настройки сохранены".to_string(),
                                        ),
                                    );
                                let mut settings_resource = settings_resource;
                                settings_resource.restart();
                            }
                            Err(e) => {
                                notification
                                    .set(
                                        Notification::Error(
                                            format!("Ошибка сохранения: {}", e),
                                        ),
                                    );
                            }
                        }
                        loading.set(false);
                    });
                },
                notification,
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

#[derive(Clone, PartialEq)]
enum Notification {
    None,
    Success(String),
    Error(String),
}

#[component]
fn ProfileContent(
    settings: UserSettings,
    on_save: EventHandler<UserSettings>,
    mut notification: Signal<Notification>,
    loading: bool,
) -> Element {
    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            // Notification area
            match notification() {
                Notification::Success(msg) => rsx! {
                    Card { class: Some("border-green-200 bg-green-50".to_string()),
                        Paragraph { class: Some("text-green-800".to_string()), "{msg}" }
                    }
                },
                Notification::Error(msg) => rsx! {
                    Card { class: Some("border-red-200 bg-red-50".to_string()),
                        Paragraph { class: Some("text-red-800".to_string()), "{msg}" }
                    }
                },
                Notification::None => rsx! {},
            }

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
        embedding: Some(settings.embedding().clone()),
        translation: Some(settings.translation().clone()),
        duolingo_jwt_token: Some(settings.duolingo_jwt_token().map(|s| s.to_string())),
    };

    UpdateUserSettingsUseCase::new(repo)
        .execute(user_id, request)
        .await
        .map_err(to_error)?;

    Ok(())
}
