mod app;
mod event_handler;
mod terminal;
mod ui;
mod widgets;

use crate::application::{
    embedding_service::EmbeddingService, llm_service::LlmService, srs_service::SrsService,
    use_cases::rate_card::RateCardUseCase,
    use_cases::start_study_session::StartStudySessionUseCase, user_repository::UserRepository,
};
use ulid::Ulid;

pub use app::AppState;
pub use terminal::{cleanup, initialize};

pub fn init_tui_app<
    R: UserRepository + 'static,
    L: LlmService + 'static,
    S: SrsService + 'static,
    E: EmbeddingService + 'static,
>(
    user_id: Ulid,
    repository: R,
    _llm_service: L,
    srs_service: S,
    _embedding_generator: E,
) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_tui(user_id, repository, srs_service))
}

async fn run_tui<R: UserRepository + 'static, S: SrsService + 'static>(
    user_id: Ulid,
    repository: R,
    srs_service: S,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = terminal::initialize()?;

    let start_study_session_use_case = StartStudySessionUseCase::new(&repository);
    let rate_card_use_case = RateCardUseCase::new(&repository, &srs_service);

    let cards = start_study_session_use_case
        .execute(user_id)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let mut app_state = app::AppState::new(user_id, cards);

    loop {
        terminal.draw(|f| ui::render(f, &app_state))?;

        if event_handler::handle_key_event(
            &mut app_state,
            &start_study_session_use_case,
            &rate_card_use_case,
        )
        .await?
        {
            break;
        }
    }

    terminal::cleanup(&mut terminal)?;
    Ok(())
}
