use iocraft::prelude::*;
use ulid::Ulid;

use crate::{
    application::{
        RateCardUseCase,
        StartStudySessionUseCase,
    },
    domain::{Card, JeersError, Rating},
    settings::Settings,
};

pub async fn handle_learn(user_id: Ulid) -> Result<(), JeersError> {
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
        let user_id = user_id;
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
        let user_id = *user_id;

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
