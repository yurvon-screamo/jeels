use crate::application::{
    EmbeddingService, srs_service::SrsService, use_cases::create_card::CreateCardUseCase,
    use_cases::delete_card::DeleteCardUseCase, use_cases::edit_card::EditCardUseCase,
    use_cases::list_cards::ListCardsUseCase, use_cases::rate_card::RateCardUseCase,
    use_cases::start_study_session::StartStudySessionUseCase, user_repository::UserRepository,
};
use crate::domain::value_objects::Rating;
use crate::tui::app::{AppState, InputField, MenuOption, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub async fn handle_key_event<
    'a,
    R: UserRepository,
    S: SrsService,
    E: EmbeddingService,
>(
    app_state: &mut AppState,
    start_study_session_use_case: &'a StartStudySessionUseCase<'a, R>,
    rate_card_use_case: &'a RateCardUseCase<'a, R, S>,
    list_cards_use_case: &'a ListCardsUseCase<'a, R>,
    create_card_use_case: &'a CreateCardUseCase<'a, R>,
    edit_card_use_case: &'a EditCardUseCase<'a, R>,
    delete_card_use_case: &'a DeleteCardUseCase<'a, R>,
    embedding_service: &mut E,
) -> Result<bool, Box<dyn std::error::Error>> {
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            return handle_key_press(
                app_state,
                key.code,
                start_study_session_use_case,
                rate_card_use_case,
                list_cards_use_case,
                create_card_use_case,
                edit_card_use_case,
                delete_card_use_case,
                embedding_service,
            )
            .await;
        }
    }
    Ok(false)
}

async fn handle_key_press<
    'a,
    R: UserRepository,
    S: SrsService,
    E: EmbeddingService,
>(
    app_state: &mut AppState,
    key_code: KeyCode,
    start_study_session_use_case: &'a StartStudySessionUseCase<'a, R>,
    rate_card_use_case: &'a RateCardUseCase<'a, R, S>,
    list_cards_use_case: &'a ListCardsUseCase<'a, R>,
    create_card_use_case: &'a CreateCardUseCase<'a, R>,
    edit_card_use_case: &'a EditCardUseCase<'a, R>,
    delete_card_use_case: &'a DeleteCardUseCase<'a, R>,
    embedding_service: &mut E,
) -> Result<bool, Box<dyn std::error::Error>> {
    match app_state.screen {
        Screen::MainMenu => handle_main_menu_key(app_state, key_code, start_study_session_use_case, list_cards_use_case).await,
        Screen::StudySession => handle_study_session_key(app_state, key_code, start_study_session_use_case, rate_card_use_case).await,
        Screen::CardList => handle_card_list_key(app_state, key_code, list_cards_use_case).await,
        Screen::CardView { .. } => handle_card_view_key(app_state, key_code, list_cards_use_case, edit_card_use_case, delete_card_use_case, embedding_service).await,
        Screen::CardEdit { .. } => handle_card_edit_key(app_state, key_code, edit_card_use_case, embedding_service).await,
        Screen::CardCreate { .. } => handle_card_create_key(app_state, key_code, create_card_use_case, embedding_service).await,
    }
}

async fn handle_main_menu_key<'a, R: UserRepository>(
    app_state: &mut AppState,
    key_code: KeyCode,
    start_study_session_use_case: &'a StartStudySessionUseCase<'a, R>,
    list_cards_use_case: &'a ListCardsUseCase<'a, R>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => Ok(true),
        KeyCode::Up => {
            app_state.prev_menu_option();
            Ok(false)
        }
        KeyCode::Down => {
            app_state.next_menu_option();
            Ok(false)
        }
        KeyCode::Enter => {
            match app_state.selected_menu_option {
                MenuOption::Study => {
                    let cards = start_study_session_use_case
                        .execute(app_state.user_id)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                    app_state.update_cards(cards);
                    app_state.screen = Screen::StudySession;
                    Ok(false)
                }
                MenuOption::ManageCards => {
                    let cards = list_cards_use_case
                        .execute(app_state.user_id)
                        .await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                    app_state.update_all_cards(cards);
                    app_state.screen = Screen::CardList;
                    Ok(false)
                }
                MenuOption::CreateCard => {
                    app_state.screen = Screen::CardCreate {
                        question: String::new(),
                        answer: String::new(),
                    };
                    app_state.input_field = InputField::Question;
                    app_state.input_mode = true;
                    Ok(false)
                }
            }
        }
        _ => Ok(false),
    }
}

async fn handle_study_session_key<'a, R: UserRepository, S: SrsService>(
    app_state: &mut AppState,
    key_code: KeyCode,
    start_study_session_use_case: &'a StartStudySessionUseCase<'a, R>,
    rate_card_use_case: &'a RateCardUseCase<'a, R, S>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app_state.screen = Screen::MainMenu;
            Ok(false)
        }
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

async fn handle_card_list_key<'a, R: UserRepository>(
    app_state: &mut AppState,
    key_code: KeyCode,
    _list_cards_use_case: &'a ListCardsUseCase<'a, R>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app_state.screen = Screen::MainMenu;
            Ok(false)
        }
        KeyCode::Up => {
            app_state.prev_card_in_list();
            Ok(false)
        }
        KeyCode::Down => {
            app_state.next_card_in_list();
            Ok(false)
        }
        KeyCode::Enter => {
            if let Some(card) = app_state.current_card_in_list() {
                app_state.screen = Screen::CardView {
                    card_id: card.id(),
                };
            }
            Ok(false)
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if let Some(card) = app_state.current_card_in_list() {
                app_state.screen = Screen::CardEdit {
                    card_id: card.id(),
                    question: card.question().text().to_string(),
                    answer: card.answer().text().to_string(),
                };
                app_state.input_field = InputField::Question;
                app_state.input_mode = true;
            }
            Ok(false)
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(card) = app_state.current_card_in_list() {
                app_state.screen = Screen::CardView {
                    card_id: card.id(),
                };
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn handle_card_view_key<
    'a,
    R: UserRepository,
    E: EmbeddingService,
>(
    app_state: &mut AppState,
    key_code: KeyCode,
    list_cards_use_case: &'a ListCardsUseCase<'a, R>,
    _edit_card_use_case: &'a EditCardUseCase<'a, R>,
    delete_card_use_case: &'a DeleteCardUseCase<'a, R>,
    _embedding_service: &mut E,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => {
            let cards = list_cards_use_case
                .execute(app_state.user_id)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            app_state.update_all_cards(cards);
            app_state.screen = Screen::CardList;
            Ok(false)
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if let Screen::CardView { card_id } = app_state.screen {
                if let Some(card) = app_state.all_cards.iter().find(|c| c.id() == card_id) {
                    app_state.screen = Screen::CardEdit {
                        card_id,
                        question: card.question().text().to_string(),
                        answer: card.answer().text().to_string(),
                    };
                    app_state.input_field = InputField::Question;
                    app_state.input_mode = true;
                }
            }
            Ok(false)
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Screen::CardView { card_id } = app_state.screen {
                delete_card_use_case
                    .execute(app_state.user_id, card_id)
                    .await?;
                let cards = list_cards_use_case
                    .execute(app_state.user_id)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                app_state.update_all_cards(cards);
                app_state.screen = Screen::CardList;
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

async fn handle_card_edit_key<
    'a,
    R: UserRepository,
    E: EmbeddingService,
>(
    app_state: &mut AppState,
    key_code: KeyCode,
    edit_card_use_case: &'a EditCardUseCase<'a, R>,
    embedding_service: &mut E,
) -> Result<bool, Box<dyn std::error::Error>> {
    if app_state.input_mode {
        match key_code {
            KeyCode::Esc => {
                app_state.input_mode = false;
                app_state.input_field = InputField::None;
                Ok(false)
            }
            KeyCode::Tab | KeyCode::Down => {
                app_state.input_field = match app_state.input_field {
                    InputField::Question => InputField::Answer,
                    InputField::Answer => InputField::Question,
                    InputField::None => InputField::Question,
                };
                Ok(false)
            }
            KeyCode::Up => {
                app_state.input_field = match app_state.input_field {
                    InputField::Question => InputField::Answer,
                    InputField::Answer => InputField::Question,
                    InputField::None => InputField::Question,
                };
                Ok(false)
            }
            KeyCode::Enter => {
                if let Screen::CardEdit { card_id, question, answer } = &app_state.screen {
                    edit_card_use_case
                        .execute(embedding_service, app_state.user_id, *card_id, question.clone(), answer.clone())
                        .await?;
                    app_state.screen = Screen::CardView { card_id: *card_id };
                    app_state.input_mode = false;
                    app_state.input_field = InputField::None;
                }
                Ok(false)
            }
            KeyCode::Backspace => {
                if let Screen::CardEdit { question, answer, .. } = &mut app_state.screen {
                    match app_state.input_field {
                        InputField::Question => {
                            question.pop();
                        }
                        InputField::Answer => {
                            answer.pop();
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            KeyCode::Char(c) => {
                if let Screen::CardEdit { question, answer, .. } = &mut app_state.screen {
                    match app_state.input_field {
                        InputField::Question => {
                            question.push(c);
                        }
                        InputField::Answer => {
                            answer.push(c);
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    } else {
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if let Screen::CardEdit { card_id, .. } = app_state.screen {
                    app_state.screen = Screen::CardView { card_id };
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

async fn handle_card_create_key<
    'a,
    R: UserRepository,
    E: EmbeddingService,
>(
    app_state: &mut AppState,
    key_code: KeyCode,
    create_card_use_case: &'a CreateCardUseCase<'a, R>,
    embedding_service: &mut E,
) -> Result<bool, Box<dyn std::error::Error>> {
    if app_state.input_mode {
        match key_code {
            KeyCode::Esc => {
                app_state.screen = Screen::MainMenu;
                app_state.input_mode = false;
                app_state.input_field = InputField::None;
                Ok(false)
            }
            KeyCode::Tab | KeyCode::Down => {
                app_state.input_field = match app_state.input_field {
                    InputField::Question => InputField::Answer,
                    InputField::Answer => InputField::Question,
                    InputField::None => InputField::Question,
                };
                Ok(false)
            }
            KeyCode::Up => {
                app_state.input_field = match app_state.input_field {
                    InputField::Question => InputField::Answer,
                    InputField::Answer => InputField::Question,
                    InputField::None => InputField::Question,
                };
                Ok(false)
            }
            KeyCode::Enter => {
                if let Screen::CardCreate { question, answer } = &app_state.screen {
                    if !question.trim().is_empty() && !answer.trim().is_empty() {
                        create_card_use_case
                            .execute(embedding_service, app_state.user_id, question.clone(), answer.clone())
                            .await?;
                        app_state.screen = Screen::MainMenu;
                        app_state.input_mode = false;
                        app_state.input_field = InputField::None;
                    }
                }
                Ok(false)
            }
            KeyCode::Backspace => {
                if let Screen::CardCreate { question, answer } = &mut app_state.screen {
                    match app_state.input_field {
                        InputField::Question => {
                            question.pop();
                        }
                        InputField::Answer => {
                            answer.pop();
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            KeyCode::Char(c) => {
                if let Screen::CardCreate { question, answer } = &mut app_state.screen {
                    match app_state.input_field {
                        InputField::Question => {
                            question.push(c);
                        }
                        InputField::Answer => {
                            answer.push(c);
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    } else {
        match key_code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app_state.screen = Screen::MainMenu;
                Ok(false)
            }
            _ => Ok(false),
        }
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
