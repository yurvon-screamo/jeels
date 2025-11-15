mod app;
mod event_handler;
mod terminal;
pub mod ui;
mod widgets;

use crate::application::{
    embedding_service::EmbeddingService, llm_service::LlmService, srs_service::SrsService,
    use_cases::create_card::CreateCardUseCase, use_cases::delete_card::DeleteCardUseCase,
    use_cases::edit_card::EditCardUseCase, use_cases::list_cards::ListCardsUseCase,
    use_cases::rate_card::RateCardUseCase,
    use_cases::start_study_session::StartStudySessionUseCase, user_repository::UserRepository,
};
use ulid::Ulid;

pub use app::AppState;
pub use terminal::{cleanup, initialize};

pub async fn init_tui_app<
    R: UserRepository + 'static,
    L: LlmService + 'static,
    S: SrsService + 'static,
    E: EmbeddingService + 'static,
>(
    user_id: Ulid,
    repository: R,
    _llm_service: L,
    srs_service: S,
    embedding_generator: E,
) -> Result<(), Box<dyn std::error::Error>> {
    run_tui(user_id, repository, srs_service, embedding_generator).await
}

async fn run_tui<
    R: UserRepository + 'static,
    S: SrsService + 'static,
    E: EmbeddingService + 'static,
>(
    user_id: Ulid,
    repository: R,
    srs_service: S,
    mut embedding_generator: E,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = terminal::initialize()?;

    let start_study_session_use_case = StartStudySessionUseCase::new(&repository);
    let rate_card_use_case = RateCardUseCase::new(&repository, &srs_service);
    let list_cards_use_case = ListCardsUseCase::new(&repository);
    let create_card_use_case = CreateCardUseCase::new(&repository);
    let edit_card_use_case = EditCardUseCase::new(&repository);
    let delete_card_use_case = DeleteCardUseCase::new(&repository);

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
            &list_cards_use_case,
            &create_card_use_case,
            &edit_card_use_case,
            &delete_card_use_case,
            &mut embedding_generator,
        )
        .await?
        {
            break;
        }
    }

    terminal::cleanup(&mut terminal)?;
    Ok(())
}
