use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Cell, Paragraph, Row, Table, Widget},
};
use ulid::Ulid;

use crate::{
    application::{CreateCardUseCase, DeleteCardUseCase, EditCardUseCase, ListCardsUseCase},
    domain::{Card, JeersError},
    settings::ApplicationEnvironment,
};

use super::render_once;

pub async fn handle_list_cards(user_id: Ulid) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;
    let cards = ListCardsUseCase::new(repository).execute(user_id).await?;

    render_once(
        |frame| {
            let area = frame.area();
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [title_area, table_area] = vertical.areas(area);

            let title = Line::from("Список карточек:".bold().underlined());
            Paragraph::new(title)
                .alignment(Alignment::Left)
                .render(title_area, frame.buffer_mut());

            render_cards_table(&cards, table_area, frame);
        },
        4 + cards.len() as u16,
    )?;

    Ok(())
}

pub async fn handle_create_card(
    user_id: Ulid,
    question: String,
    answer: String,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let card = CreateCardUseCase::new(
        settings.get_repository().await?,
        settings.get_embedding_generator().await?,
        settings.get_llm_service().await?,
    )
    .execute(user_id, question, Some(answer))
    .await?;

    render_once(
        |frame| {
            let area = frame.area();
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [title_area, card_area] = vertical.areas(area);

            let title = Line::from("Создана карточка:".bold().underlined());
            Paragraph::new(title)
                .alignment(Alignment::Left)
                .render(title_area, frame.buffer_mut());

            render_card(&card, card_area, frame);
        },
        10,
    )?;

    Ok(())
}

pub async fn handle_create_words(user_id: Ulid, questions: Vec<String>) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let use_case = CreateCardUseCase::new(
        settings.get_repository().await?,
        settings.get_embedding_generator().await?,
        settings.get_llm_service().await?,
    );

    for question in questions {
        let card = use_case.execute(user_id, question, None).await?;
        render_once(
            |frame| {
                let area = frame.area();
                let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
                let [title_area, card_area] = vertical.areas(area);

                let title = Line::from("Создана карточка:".bold().underlined());
                Paragraph::new(title)
                    .alignment(Alignment::Left)
                    .render(title_area, frame.buffer_mut());

                render_card(&card, card_area, frame);
            },
            10,
        )?;
    }

    Ok(())
}

pub async fn handle_edit_card(
    user_id: Ulid,
    card_id: Ulid,
    question: String,
    answer: String,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let card = EditCardUseCase::new(
        settings.get_repository().await?,
        settings.get_embedding_generator().await?,
    )
    .execute(user_id, card_id, question, answer)
    .await?;

    render_once(
        |frame| {
            let area = frame.area();
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [title_area, card_area] = vertical.areas(area);

            let title = Line::from("Карточка отредактирована:".bold().underlined());
            Paragraph::new(title)
                .alignment(Alignment::Left)
                .render(title_area, frame.buffer_mut());

            render_card(&card, card_area, frame);
        },
        10,
    )?;

    Ok(())
}

pub async fn handle_delete_card(user_id: Ulid, card_ids: Vec<Ulid>) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();

    for card_id in card_ids {
        let card = DeleteCardUseCase::new(settings.get_repository().await?)
            .execute(user_id, card_id)
            .await?;

        render_once(
            |frame| {
                let area = frame.area();
                let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
                let [title_area, card_area] = vertical.areas(area);

                let title = Line::from("Карточка удалена:".bold().underlined());
                Paragraph::new(title)
                    .alignment(Alignment::Left)
                    .render(title_area, frame.buffer_mut());

                render_card(&card, card_area, frame);
            },
            10,
        )?;
    }

    Ok(())
}

fn render_card(card: &Card, area: Rect, frame: &mut Frame) {
    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Min(0),
    ]);
    let [id_area, qa_area, stats_area] = vertical.areas(area);

    // ID block
    let id_block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Blue));
    let id_text = Text::from(vec![Line::from(format!("Карточка с ID: {}", card.id()))]);
    Paragraph::new(id_text)
        .block(id_block)
        .render(id_area, frame.buffer_mut());

    // Question/Answer block
    let qa_block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Blue));
    let qa_text = Text::from(vec![
        Line::from(format!("Вопрос: {}", card.question().text())),
        Line::from(format!("Ответ: {}", card.answer().text())),
    ]);
    Paragraph::new(qa_text)
        .block(qa_block)
        .render(qa_area, frame.buffer_mut());

    // Stats block
    if card.memory_state().is_some() {
        let stats_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Blue));
        let memory_state_text = card
            .memory_state()
            .map(|state| format!("{:.2}", state.difficulty()))
            .unwrap_or_else(|| "None".to_string());
        let stats_text = Text::from(vec![
            Line::from(format!("Оценок: {}", card.reviews().len())),
            Line::from(format!(
                "Дата следующего повторения: {}",
                card.next_review_date()
            )),
            Line::from(format!("Стабильность: {}", card.stability())),
            Line::from(format!("Состояние памяти: {}", memory_state_text)),
        ]);
        Paragraph::new(stats_text)
            .block(stats_block)
            .render(stats_area, frame.buffer_mut());
    }
}

fn render_cards_table(cards: &[Card], area: Rect, frame: &mut Frame) {
    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan));

    if cards.is_empty() {
        let empty_text = Text::from("Нет карточек");
        Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Center)
            .render(area, frame.buffer_mut());
        return;
    }

    let header = Row::new(vec![
        Cell::from("Id".bold().underlined()),
        Cell::from("Вопрос".bold().underlined()),
        Cell::from("Ответ".bold().underlined()),
        Cell::from("Оценок".bold().underlined()),
        Cell::from("Дата следующего повторения".bold().underlined()),
    ])
    .style(Style::default());

    let rows: Vec<Row> = cards
        .iter()
        .enumerate()
        .map(|(i, card)| {
            let style = if i % 2 == 0 {
                Style::default()
            } else {
                Style::default().bg(Color::DarkGray)
            };
            Row::new(vec![
                Cell::from(card.id().to_string()),
                Cell::from(card.question().text().to_string()),
                Cell::from(card.answer().text().to_string()),
                Cell::from(card.reviews().len().to_string()),
                Cell::from(card.next_review_date().to_string()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(15),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(10),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(1);

    table.render(area, frame.buffer_mut());
}
