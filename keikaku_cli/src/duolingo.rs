use ulid::Ulid;

use crate::cli::render_once;
use keikaku::{
    application::{SyncDuolingoWordsUseCase, UserRepository},
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

pub async fn handle_sync_duolingo_words(
    user_id: Ulid,
    question_only: bool,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;

    let user = repository
        .find_by_id(user_id)
        .await?
        .ok_or(JeersError::UserNotFound { user_id })?;

    if user.settings().duolingo_jwt_token().is_none() {
        return Err(JeersError::RepositoryError {
            reason: "Duolingo JWT token not set. Please set it first.".to_string(),
        });
    }

    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(ratatui::style::Style::default().fg(Color::Yellow));
            let text = ratatui::text::Text::from(vec![Line::from(
                "Синхронизация слов из Duolingo...".fg(Color::Yellow),
            )]);
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .render(area, frame.buffer_mut());
        },
        5,
    )?;

    let duolingo_client = keikaku::infrastructure::HttpDuolingoClient::new();
    let llm_service = settings.get_llm_service(user_id).await?;
    let use_case = SyncDuolingoWordsUseCase::new(
        settings.get_repository().await?,
        &llm_service,
        &duolingo_client,
    );

    let result = use_case.execute(user_id, question_only).await?;

    let mut text_lines = vec![
        Line::from("Синхронизация завершена успешно!".bold().fg(Color::Green)),
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
