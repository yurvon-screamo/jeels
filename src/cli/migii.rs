use ulid::Ulid;

use crate::{
    application::{ExportMigiiPackUseCase, UserRepository},
    cli::render_once,
    domain::JeersError,
    settings::ApplicationEnvironment,
};
use ratatui::{
    layout::Alignment,
    style::{Color, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

pub async fn handle_create_migii_pack(
    user_id: Ulid,
    lessons: Vec<u32>,
    question_only: bool,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;

    let user = repository
        .find_by_id(user_id)
        .await?
        .ok_or(JeersError::UserNotFound { user_id })?;

    let level = user.current_japanese_level();

    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(ratatui::style::Style::default().fg(Color::Yellow));
            let text = ratatui::text::Text::from(vec![Line::from(
                format!(
                    "Создание пачки из {} уроков уровня {:?}...",
                    lessons.len(),
                    level
                )
                .fg(Color::Yellow),
            )]);
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .render(area, frame.buffer_mut());
        },
        5,
    )?;

    let use_case = ExportMigiiPackUseCase::new(
        settings.get_repository().await?,
        settings.get_embedding_service().await?,
        settings.get_llm_service().await?,
        settings.get_migii_client().await?,
    );

    let result = use_case.execute(user_id, lessons, question_only).await?;

    let mut text_lines = vec![
        Line::from("Пачка создана успешно!".bold().fg(Color::Green)),
        Line::from(""),
        Line::from(format!("Создано карточек: {}", result.total_created_count)),
        Line::from(format!(
            "Пропущено (дубликаты): {}",
            result.skipped_words.len()
        )),
    ];

    if !result.skipped_words.is_empty() {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from("Пропущенные слова:".bold().fg(Color::Yellow)));
        for word in &result.skipped_words {
            text_lines.push(Line::from(format!("  • {}", word).fg(Color::Gray)));
        }
    }

    let height = (text_lines.len() + 2) as u16;
    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(ratatui::style::Style::default().fg(Color::Green));
            Paragraph::new(ratatui::text::Text::from(text_lines))
                .block(block)
                .alignment(Alignment::Left)
                .render(area, frame.buffer_mut());
        },
        height,
    )?;

    Ok(())
}
