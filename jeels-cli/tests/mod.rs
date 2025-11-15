use jeels_cli::application::UserRepository;
use jeels_cli::domain::User;
use jeels_cli::infrastructure::SurrealUserRepository;
use jeels_cli::settings::{AuthSettings, DatabaseSettings, LlmSettings, Settings};
use tempfile::TempDir;

pub struct TestContext {
    pub repository: SurrealUserRepository,
    pub _temp_dir: TempDir,
}

pub async fn create_test_repository() -> TestContext {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");

    let settings = Settings {
        llm: LlmSettings {
            model_path: "qwen3-0.6b.gguf".into(),
        },
        database: DatabaseSettings {
            path: db_path,
            namespace: "test".to_string(),
            database: "test".to_string(),
            auth: AuthSettings {
                username: "root".to_string(),
                password: "root".to_string(),
            },
        },
    };

    let repository = SurrealUserRepository::new(&settings).await.unwrap();

    TestContext {
        repository,
        _temp_dir: temp_dir,
    }
}

pub async fn create_test_user(repository: &SurrealUserRepository) -> User {
    let user = User::new("test_user".to_string());
    repository.save(&user).await.unwrap();
    user
}
