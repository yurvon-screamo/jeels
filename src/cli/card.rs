use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use crate::{
    application::{
        CreateCardUseCase, DeleteCardUseCase, EditCardUseCase, ListCardsUseCase,
        RebuildDatabaseUseCase,
    },
    domain::{JeersError, VocabularyCard},
    settings::ApplicationEnvironment,
};

use super::render_once;

pub async fn handle_list_cards(user_id: Ulid) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;
    let cards = ListCardsUseCase::new(repository).execute(user_id).await?;

    println!("Список карточек:");
    println!();

    if cards.is_empty() {
        println!("Нет карточек");
        return Ok(());
    }

    // Header
    println!(
        "{:<20} | {:<20} | {:<50} | {:>2} | {:<19}",
        "Id", "Вопрос", "Ответ", "Оценок", "След. повторение"
    );
    println!("{}", "-".repeat(120));

    // Data rows
    for card in cards.iter() {
        let id_str = truncate_text(&card.id().to_string(), 20);
        let question_str = truncate_text(card.question().text(), 20);
        let answer_str = truncate_text(card.answer().text(), 50);
        let reviews_str = card.reviews().len().to_string();
        let date = card
            .next_review_date()
            .naive_local()
            .format("%Y-%m-%d %H:%M");
        let date_str = truncate_text(&date.to_string(), 19);

        let row = format!(
            "{:<20} | {:<20} | {:<50} | {:>2} | {:<19}",
            id_str, question_str, answer_str, reviews_str, date_str
        );

        if card.is_new() {
            // Highlight new cards in cyan (priority over is_due)
            println!("\x1b[96m{}\x1b[0m", row);
        } else if card.is_due() {
            // Highlight cards ready for review in yellow
            println!("\x1b[93m{}\x1b[0m", row);
        } else {
            println!("{}", row);
        }
    }

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
    .execute(user_id, question, Some(answer), None)
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
        let card = use_case.execute(user_id, question, None, None).await?;
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
    .execute(user_id, card_id, question, answer, Vec::new())
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

pub async fn handle_rebuild_database(
    user_id: Ulid,
    rebuild_example_phrases: bool,
    rebuild_embedding: bool,
    rebuild_answer: bool,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;
    let embedding_service = settings.get_embedding_generator().await?;
    let create_card_use_case = CreateCardUseCase::new(
        repository,
        embedding_service,
        settings.get_llm_service().await?,
    );
    let rebuild_use_case =
        RebuildDatabaseUseCase::new(repository, embedding_service, create_card_use_case);
    let processed_count = rebuild_use_case
        .execute(
            user_id,
            rebuild_example_phrases,
            rebuild_embedding,
            rebuild_answer,
        )
        .await?;

    render_once(
        |frame| {
            let area = frame.area();
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [title_area, content_area] = vertical.areas(area);

            let title = Line::from("Пересборка базы данных завершена:".bold().underlined());
            Paragraph::new(title)
                .alignment(Alignment::Left)
                .render(title_area, frame.buffer_mut());

            let content = vec![Line::from(format!(
                "Обработано карточек: {}",
                processed_count
            ))];

            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Green));

            Paragraph::new(Text::from(content))
                .block(block)
                .render(content_area, frame.buffer_mut());
        },
        5,
    )?;

    Ok(())
}

fn render_card(card: &VocabularyCard, area: Rect, frame: &mut Frame) {
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
    if let Some(stability) = card.stability()
        && let Some(difficulty) = card.difficulty()
    {
        let stats_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Blue));
        let stats_text = Text::from(vec![
            Line::from(format!("Оценок: {}", card.reviews().len())),
            Line::from(format!(
                "Дата следующего повторения: {}",
                card.next_review_date()
            )),
            Line::from(format!("Стабильность: {}", stability.value())),
            Line::from(format!("Сложность: {}", difficulty.value())),
        ]);
        Paragraph::new(stats_text)
            .block(stats_block)
            .render(stats_area, frame.buffer_mut());
    }
}

fn truncate_text(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len - 3).collect::<String>() + "..."
    }
}
