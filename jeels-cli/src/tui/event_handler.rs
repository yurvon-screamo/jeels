use crate::application::{
    srs_service::SrsService, use_cases::rate_card::RateCardUseCase,
    use_cases::start_study_session::StartStudySessionUseCase, user_repository::UserRepository,
};
use crate::domain::value_objects::Rating;
use crate::tui::app::AppState;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub async fn handle_key_event<'a, R: UserRepository, S: SrsService>(
    app_state: &mut AppState,
    start_study_session_use_case: &'a StartStudySessionUseCase<'a, R>,
    rate_card_use_case: &'a RateCardUseCase<'a, R, S>,
) -> Result<bool, Box<dyn std::error::Error>> {
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            return handle_key_press(
                app_state,
                key.code,
                start_study_session_use_case,
                rate_card_use_case,
            )
            .await;
        }
    }
    Ok(false)
}

async fn handle_key_press<'a, R: UserRepository, S: SrsService>(
    app_state: &mut AppState,
    key_code: KeyCode,
    start_study_session_use_case: &'a StartStudySessionUseCase<'a, R>,
    rate_card_use_case: &'a RateCardUseCase<'a, R, S>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => Ok(true),
        KeyCode::Char(' ') => {
            app_state.toggle_answer();
            Ok(false)
        }
        KeyCode::Char('1') => {
            rate_and_reload(
                app_state,
                Rating::Again,
                start_study_session_use_case,
                rate_card_use_case,
            )
            .await?;
            Ok(false)
        }
        KeyCode::Char('2') => {
            rate_and_reload(
                app_state,
                Rating::Hard,
                start_study_session_use_case,
                rate_card_use_case,
            )
            .await?;
            Ok(false)
        }
        KeyCode::Char('3') => {
            rate_and_reload(
                app_state,
                Rating::Good,
                start_study_session_use_case,
                rate_card_use_case,
            )
            .await?;
            Ok(false)
        }
        KeyCode::Char('4') => {
            rate_and_reload(
                app_state,
                Rating::Easy,
                start_study_session_use_case,
                rate_card_use_case,
            )
            .await?;
            Ok(false)
        }
        KeyCode::Char('n') | KeyCode::Right => {
            app_state.next_card();
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn rate_and_reload<'a, R: UserRepository, S: SrsService>(
    app_state: &mut AppState,
    rating: Rating,
    start_study_session_use_case: &'a StartStudySessionUseCase<'a, R>,
    rate_card_use_case: &'a RateCardUseCase<'a, R, S>,
) -> Result<(), Box<dyn std::error::Error>> {
    if app_state.cards.is_empty() {
        return Ok(());
    }

    let card_id = app_state.get_current_card_id();
    rate_card_use_case
        .execute(app_state.user_id, card_id, rating)
        .await?;

    let cards = start_study_session_use_case
        .execute(app_state.user_id)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    app_state.update_cards(cards);

    Ok(())
}
