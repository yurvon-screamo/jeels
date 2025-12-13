use tokio::sync::OnceCell;
use ulid::Ulid;

use keikaku::{
    application::UserRepository,
    domain::{
        value_objects::{JapaneseLevel, NativeLanguage},
        User,
    },
    settings::ApplicationEnvironment,
};

pub const DEFAULT_USERNAME: &str = "yurvon_screamo";

static ENV_INIT: OnceCell<Result<(), String>> = OnceCell::const_new();

pub async fn init_env() -> Result<&'static ApplicationEnvironment, String> {
    let res = ENV_INIT
        .get_or_init(|| async { ApplicationEnvironment::load().await.map_err(to_error) })
        .await;
    match res {
        Ok(_) => Ok(ApplicationEnvironment::get()),
        Err(err) => Err(err.clone()),
    }
}

pub async fn ensure_user(
    env: &'static ApplicationEnvironment,
    username: &str,
) -> Result<Ulid, String> {
    let repo = env.get_repository().await.map_err(to_error)?;
    if let Some(user) = repo.find_by_username(username).await.map_err(to_error)? {
        return Ok(user.id());
    }
    let new_user = User::new(
        username.to_string(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
        7,
    );
    let id = new_user.id();
    repo.save(&new_user).await.map_err(to_error)?;
    Ok(id)
}

pub fn to_error(err: impl std::fmt::Display) -> String {
    err.to_string()
}
