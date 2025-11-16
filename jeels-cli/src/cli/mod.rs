use clap::Parser;
use iocraft::prelude::*;
use ulid::Ulid;

use crate::{
    application::{
        CreateCardUseCase, DeleteCardUseCase, EditCardUseCase, ListCardsUseCase, RateCardUseCase,
        StartStudySessionUseCase, UserRepository,
    },
    domain::{Card, JapaneseLevel, JeersError, NativeLanguage, Rating, User},
    settings::Settings,
};

const DEFAULT_USERNAME: &str = "yurvon_screamo";
const DEFAULT_JAPANESE_LEVEL: JapaneseLevel = JapaneseLevel::N5;
const DEFAULT_NATIVE_LANGUAGE: NativeLanguage = NativeLanguage::Russian;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = DEFAULT_USERNAME)]
    username: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Me {},
    Learn {},
    Cards {},
    Create {
        question: String,
        answer: String,
    },
    Edit {
        card_id: Ulid,
        question: String,
        answer: String,
    },
    Delete {
        card_id: Ulid,
    },
}

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let user_id = ensure_user_exists(Settings::get(), &args.username).await?;

    match args.command {
        Command::Me {} => {
            handle_me(user_id).await?;
        }
        Command::Cards {} => {
            handle_list_cards(user_id).await?;
        }
        Command::Learn {} => {
            handle_learn(user_id).await?;
        }
        Command::Create { question, answer } => {
            handle_create_card(user_id, question, answer).await?;
        }
        Command::Edit {
            card_id,
            question,
            answer,
        } => {
            handle_edit_card(user_id, card_id, question, answer).await?;
        }
        Command::Delete { card_id } => {
            handle_delete_card(user_id, card_id).await?;
        }
    }

    Ok(())
}

async fn ensure_user_exists(
    settings: &'static Settings,
    username: &str,
) -> Result<Ulid, Box<dyn std::error::Error>> {
    let repository = settings.get_repository();

    if let Some(user) = repository
        .find_by_username(username)
        .await
        .map_err(|e| format!("Failed to find user: {}", e))?
    {
        Ok(user.id())
    } else {
        let new_user = User::new(
            username.to_string(),
            DEFAULT_JAPANESE_LEVEL,
            DEFAULT_NATIVE_LANGUAGE,
        );
        let user_id = new_user.id();
        repository
            .save(&new_user)
            .await
            .map_err(|e| format!("Failed to save user: {}", e))?;
        Ok(user_id)
    }
}

async fn handle_learn(user_id: Ulid) -> Result<(), JeersError> {
    let settings = Settings::get();

    let start_study_usecase = StartStudySessionUseCase::new(settings.get_repository());
    let cards = start_study_usecase.execute(user_id).await?;

    if cards.is_empty() {
        element! {
            View(
                flex_direction: FlexDirection::Column,
                margin_top: 1,
                margin_bottom: 1,
                border_style: BorderStyle::Round,
                border_color: Color::Red)
            {
                Text(content: "Вы всё выучили!", weight: Weight::Bold, color: Some(Color::Red))
            }
        }
        .print();

        return Ok(());
    }

    for card in cards {
        let user_id = user_id.clone();
        smol::block_on(
            element!(
                ContextProvider(value: Context::owned(card))
                {
                    ContextProvider(value: Context::owned(user_id)) {
                        LearnCard
                    }
                }
            )
            .render_loop(),
        )
        .map_err(|e| JeersError::RepositoryError {
            reason: e.to_string(),
        })?;
    }

    Ok(())
}

#[component]
fn LearnCard<'a>(mut hooks: Hooks) -> impl Into<AnyElement<'a>> {
    let settings = Settings::get();
    let rate_usecase = RateCardUseCase::new(settings.get_repository(), settings.get_srs_service());

    let mut system = hooks.use_context_mut::<SystemContext>();
    let card = hooks.use_context::<Card>();
    let user_id = hooks.use_context::<Ulid>();

    let mut rate = hooks.use_state(|| None);
    let mut show_answer = hooks.use_state(|| false);
    let mut should_exit = hooks.use_state(|| false);

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char(' ') => show_answer.set(true),
                    KeyCode::Char('s') => should_exit.set(true),
                    KeyCode::Char('1') => rate.set(Some(Rating::Easy)),
                    KeyCode::Char('2') => rate.set(Some(Rating::Good)),
                    KeyCode::Char('3') => rate.set(Some(Rating::Hard)),
                    KeyCode::Char('4') => rate.set(Some(Rating::Again)),
                    _ => {}
                }
            }
            _ => {}
        }
    });

    if should_exit.get() {
        system.exit();
    }

    if let Some(current_rate) = rate.get() {
        let card_id = card.id();
        let user_id = user_id.clone();

        if let Err(e) = smol::block_on(rate_usecase.execute(user_id, card_id, current_rate)) {
            eprintln!("Error rating card: {:?}", e);
        }

        should_exit.set(true);
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            border_style: BorderStyle::Round,
            border_color: Color::Green,
            width: 60,
            margin_left: 2
        ) {
            #(if should_exit.get() {
                element! {
                    View(flex_direction: FlexDirection::Column) {
                        View { Text(content: card.question().text(), weight: Weight::Bold, color: Some(Color::Blue))}
                        View { Text(content: card.answer().text(), weight: Weight::Bold, color: Some(Color::Magenta))}
                    }
                }
            } else if show_answer.get() {
                element! {
                    View(flex_direction: FlexDirection::Column) {
                        View { Text(content: card.question().text(), weight: Weight::Bold, color: Some(Color::Blue))}
                        View { Text(content: card.answer().text(), weight: Weight::Bold, color: Some(Color::Magenta)) }
                        View(margin_top: 1, flex_direction: FlexDirection::Column) {
                            Text(content: "Используйте цифры от 1 до 4 для оценки карточки.", weight: Weight::Light, color: Some(Color::Grey))
                            Text(content: "1 - Легко", weight: Weight::Light, color: Some(Color::Grey))
                            Text(content: "2 - Нормально", weight: Weight::Light, color: Some(Color::Grey))
                            Text(content: "3 - Трудно", weight: Weight::Light, color: Some(Color::Grey))
                            Text(content: "4 - Очень трудно", weight: Weight::Light, color: Some(Color::Grey))
                            Text(content: "Нажмите \"s\" чтобы пропустить карточку.", weight: Weight::Light, color: Some(Color::Grey))
                        }
                    }
                }
            } else {
                element! {
                    View(flex_direction: FlexDirection::Column) {
                        View { Text(content: card.question().text(), weight: Weight::Bold, color: Some(Color::Magenta))}
                        View(margin_top: 1, flex_direction: FlexDirection::Column) {
                            Text(content: "Нажмите пробел чтобы показать ответ.", weight: Weight::Light, color: Some(Color::Grey))
                            Text(content: "Нажмите \"s\" чтобы пропустить карточку.", weight: Weight::Light, color: Some(Color::Grey))
                        }
                    }
                }
            })
        }
    }
}

async fn handle_me(user_id: Ulid) -> Result<(), JeersError> {
    let settings = Settings::get();
    let repository = settings.get_repository();
    let user = repository
        .find_by_id(user_id)
        .await?
        .ok_or(JeersError::UserNotFound { user_id })?;
    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: 1,
            margin_bottom: 1,
        ) {
            Text(content: "Информация о пользователе:", weight: Weight::Bold, decoration: TextDecoration::Underline)
            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color: Color::Magenta,
            ) {
                Text(content: format!("ID: {}", user.id()))
                Text(content: format!("Имя пользователя: {}", user.username()))
                Text(content: format!("Уровень японского: {:?}", user.current_japanese_level()))
                Text(content: format!("Родной язык: {:?}", user.native_language()))
            }
        }
    }
    .print();
    Ok(())
}

async fn handle_list_cards(user_id: Ulid) -> Result<(), JeersError> {
    let settings = Settings::get();
    let repository = settings.get_repository();
    let cards = ListCardsUseCase::new(repository).execute(user_id).await?;
    element!(
        View(
            flex_direction: FlexDirection::Column,
            margin_top: 1,
            margin_bottom: 1,
        ) {
            Text(content: "Список карточек:", weight: Weight::Bold, decoration: TextDecoration::Underline)
            ContextProvider(value: Context::owned(cards)) {
                CardsTable
            }
        }
    )
    .print();
    Ok(())
}

async fn handle_create_card(
    user_id: Ulid,
    question: String,
    answer: String,
) -> Result<(), JeersError> {
    let settings = Settings::get();
    let card = CreateCardUseCase::new(
        settings.get_repository(),
        settings.get_embedding_generator(),
    )
    .execute(user_id, question, answer)
    .await?;

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: 1,
            margin_bottom: 1,
        ) {
            Text(content: "Создана карточка:", weight: Weight::Bold, decoration: TextDecoration::Underline)
            ContextProvider(value: Context::owned(card)) {
                CardDisplay
            }
        }
    }
    .print();

    Ok(())
}

async fn handle_edit_card(
    user_id: Ulid,
    card_id: Ulid,
    question: String,
    answer: String,
) -> Result<(), JeersError> {
    let settings = Settings::get();
    let card = EditCardUseCase::new(
        settings.get_repository(),
        settings.get_embedding_generator(),
    )
    .execute(user_id, card_id, question, answer)
    .await?;

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: 1,
            margin_bottom: 1,
        ) {
            Text(content: "Карточка отредактирована:", weight: Weight::Bold, decoration: TextDecoration::Underline)
            ContextProvider(value: Context::owned(card)) {
                CardDisplay
            }
        }
    }
    .print();

    Ok(())
}

async fn handle_delete_card(user_id: Ulid, card_id: Ulid) -> Result<(), JeersError> {
    let settings = Settings::get();
    let card = DeleteCardUseCase::new(settings.get_repository())
        .execute(user_id, card_id)
        .await?;

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: 1,
            margin_bottom: 1,
        ) {
            Text(content: "Карточка удалена:", weight: Weight::Bold, decoration: TextDecoration::Underline)
            ContextProvider(value: Context::owned(card)) {
                CardDisplay
            }
        }
    }
    .print();

    Ok(())
}

#[component]
fn CardDisplay<'a>(hooks: Hooks) -> impl Into<AnyElement<'a>> {
    let card = hooks.use_context::<Card>();

    element! {
        View(
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            margin: 2,
        ){
            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
            ) {
                Text(content: format!("Карточка с ID: {}", card.id()))
            }

            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
            ) {
                Text(content: format!("Вопрос: {}", card.question().text()))
                Text(content: format!("Ответ: {}", card.answer().text()))
            }

            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color: Color::Blue,
            ) {
                Text(content: format!("Оценок: {}", card.reviews().len()))
                Text(content: format!("Дата следующего повторения: {}", card.next_review_date().to_string()))
                Text(content: format!("Стабильность: {}", card.stability().to_string()))
                Text(content: format!("Состояние памяти: {}", card.memory_state().map(|state| format!("{:.2}", state.difficulty())).unwrap_or("None".to_string())))
            }
        }
    }
}

#[component]
fn CardsTable<'a>(hooks: Hooks) -> impl Into<AnyElement<'a>> {
    let cards = hooks.use_context::<Vec<Card>>();

    element! {
        View(
            margin_top: 1,
            margin_bottom: 1,
            flex_direction: FlexDirection::Column,
            width: 160,
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            View(border_style: BorderStyle::Single, border_edges: Edges::Bottom, border_color: Color::Grey) {

                View(width: 50pct) {
                    Text(content: "Id", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 30pct) {
                    Text(content: "Вопрос", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 30pct) {
                    Text(content: "Ответ", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 20pct) {
                    Text(content: "Оценок", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }

                View(width: 50pct) {
                    Text(content: "Дата следующего повторения", weight: Weight::Bold, decoration: TextDecoration::Underline)
                }
            }

            #(cards.iter().enumerate().map(|(i, card)| element! {
                View(background_color: if i % 2 == 0 { None } else { Some(Color::DarkGrey) }) {
                    View(width: 50pct) {
                        Text(content: card.id().to_string())
                    }

                    View(width: 30pct) {
                        Text(content: card.question().text().clone())
                    }

                    View(width: 30pct) {
                        Text(content: card.answer().text().clone())
                    }

                    View(width: 20pct) {
                        Text(content: card.reviews().len().to_string())
                    }

                    View(width: 50pct) {
                        Text(content: card.next_review_date().to_string())
                    }
                }
            }))
        }
    }
}
