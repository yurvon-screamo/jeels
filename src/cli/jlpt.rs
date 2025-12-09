use ulid::Ulid;

use crate::{
    application::{ExportJlptRecommendedUseCase, UserRepository},
    cli::render_once,
    domain::{JeersError, value_objects::JapaneseLevel},
    settings::ApplicationEnvironment,
};
use ratatui::{
    layout::Alignment,
    style::{Color, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

pub async fn handle_export_jlpt_recommended(
    user_id: Ulid,
    levels: Vec<JapaneseLevel>,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;

    let user = repository
        .find_by_id(user_id)
        .await?
        .ok_or(JeersError::UserNotFound { user_id })?;

    let selected_levels = if levels.is_empty() {
        vec![user.current_japanese_level().clone()]
    } else {
        levels
    };

    let levels_label = selected_levels
        .iter()
        .map(|l| l.code().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(ratatui::style::Style::default().fg(Color::Yellow));
            let text = ratatui::text::Text::from(vec![Line::from(
                format!("Создание JLPT-пачки для уровней {levels_label}...").fg(Color::Yellow),
            )]);
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .render(area, frame.buffer_mut());
        },
        5,
    )?;

    let use_case = ExportJlptRecommendedUseCase::new(
        settings.get_repository().await?,
        settings.get_llm_service().await?,
    );

    let result = use_case.execute(user_id, selected_levels.clone()).await?;

    let mut text_lines = vec![
        Line::from("Пачка создана успешно!".bold().fg(Color::Green)),
        Line::from(""),
        Line::from(format!("Уровни: {}", levels_label)),
        Line::from(format!("Создано карточек: {}", result.total_created_count)),
        Line::from(format!(
            "Пропущено (дубликаты/ошибки): {}",
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
