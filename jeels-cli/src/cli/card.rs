use iocraft::prelude::*;
use ulid::Ulid;

use crate::{
    application::{CreateCardUseCase, DeleteCardUseCase, EditCardUseCase, ListCardsUseCase},
    domain::{Card, JeersError},
    settings::Settings,
};

pub async fn handle_list_cards(user_id: Ulid) -> Result<(), JeersError> {
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

pub async fn handle_create_card(
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

pub async fn handle_edit_card(
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

pub async fn handle_delete_card(user_id: Ulid, card_id: Ulid) -> Result<(), JeersError> {
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
