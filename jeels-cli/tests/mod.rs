use jeels_cli::application::UserRepository;
use jeels_cli::domain::{JapaneseLevel, NativeLanguage, User};
use jeels_cli::settings::ApplicationEnvironment;
use tempfile::TempDir;

pub struct TestContext {}

pub async fn create_test_repository() -> TestContext {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    // Ignore error if settings already initialized (for parallel test execution)
    let _ = ApplicationEnvironment::from_database_path(db_path);
    TestContext {}
}

pub async fn create_test_user() -> User {
    let repository = ApplicationEnvironment::get()
        .get_repository()
        .await
        .unwrap();
    let user = User::new(
        "test_user".to_string(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );
    repository.save(&user).await.unwrap();
    user
}
