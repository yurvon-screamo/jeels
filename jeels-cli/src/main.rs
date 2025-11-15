use jeels_cli::application::UserRepository;
use jeels_cli::domain::User;
use jeels_cli::infrastructure::{
    EmbeddingGenerator, FsrsSrsService, QwenLlm, SurrealUserRepository,
};
use jeels_cli::settings::Settings;
use ulid::Ulid;

const DEFAULT_USERNAME: &str = "default";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load().map_err(|e| format!("Failed to load settings: {}", e))?;
    let repository = SurrealUserRepository::new(&settings)
        .await
        .map_err(|e| format!("Failed to initialize repository: {}", e))?;
    let srs_service =
        FsrsSrsService::new().map_err(|e| format!("Failed to initialize SRS service: {}", e))?;
    let embedding_generator = EmbeddingGenerator::new()
        .map_err(|e| format!("Failed to initialize embedding generator: {}", e))?;

    let model_path = settings.llm.model_path.to_string_lossy().to_string();

    let llm_service = if settings.llm.model_path.exists() {
        QwenLlm::new(&model_path).map_err(|e| format!("Failed to initialize LLM service: {}", e))?
    } else {
        eprintln!(
            "Warning: LLM model file not found at '{}'. Generation feature will be disabled.",
            model_path
        );
        eprintln!(
            "Please download the model from https://huggingface.co/MaziyarPanahi/Qwen3-0.6B-GGUF"
        );
        eprintln!("And update the model_path in config.toml file.");
        return Err(format!(
            "LLM model not found. Please download the model and update config.toml."
        )
        .into());
    };

    let user_id = ensure_user_exists(&repository, DEFAULT_USERNAME)
        .await
        .map_err(|e| format!("Failed to ensure user exists: {}", e))?;

    jeels_cli::tui::init_tui_app(
        user_id,
        repository,
        llm_service,
        srs_service,
        embedding_generator,
    )?;

    Ok(())
}

async fn ensure_user_exists<R: UserRepository>(
    repository: &R,
    username: &str,
) -> Result<Ulid, Box<dyn std::error::Error>> {
    if let Some(user) = repository
        .find_by_username(username)
        .await
        .map_err(|e| format!("Failed to find user: {}", e))?
    {
        Ok(user.id())
    } else {
        let new_user = User::new(username.to_string());
        let user_id = new_user.id();
        repository
            .save(&new_user)
            .await
            .map_err(|e| format!("Failed to save user: {}", e))?;
        Ok(user_id)
    }
}
