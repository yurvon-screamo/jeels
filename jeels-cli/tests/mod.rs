use jeels_cli::application::UserRepository;
use jeels_cli::domain::{JapaneseLevel, NativeLanguage, User};
use jeels_cli::settings::Settings;
use tempfile::TempDir;

pub struct TestContext {}

pub async fn create_test_repository() -> TestContext {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    Settings::from_database_path(db_path).unwrap();
    TestContext {}
}

pub async fn create_test_user() -> User {
    let repository = Settings::get().get_repository();
    let user = User::new(
        "test_user".to_string(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );
    repository.save(&user).await.unwrap();
    user
}
