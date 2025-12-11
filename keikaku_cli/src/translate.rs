use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use keikaku::{application::TranslateUseCase, domain::JeersError, settings::ApplicationEnvironment};

use super::render_once;

pub async fn handle_translate(user_id: Ulid, text: String) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;
    let translation_service = settings.get_translation_service().await?;

    let use_case = TranslateUseCase::new(repository, translation_service);
    let result = use_case.execute(user_id, text.clone()).await?;

    let source_text = text.clone();
    let result_text = result.clone();

    render_once(
        |frame| {
            let area = frame.area();
            let vertical = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Min(0),
            ]);
            let [title_area, source_area, result_area, _] = vertical.areas(area);

            let title = Line::from("Перевод:".bold().underlined());
            Paragraph::new(title)
                .alignment(Alignment::Left)
                .render(title_area, frame.buffer_mut());

            let source_block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Исходный текст");

            let source_content = Text::from(vec![Line::from(source_text.as_str())]);
            Paragraph::new(source_content)
                .block(source_block)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .render(source_area, frame.buffer_mut());

            let result_block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Green))
                .title("Перевод");

            let result_content = Text::from(vec![Line::from(result_text.as_str())]);
            Paragraph::new(result_content)
                .block(result_block)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .render(result_area, frame.buffer_mut());
        },
        20,
    )
    .map_err(|e| JeersError::SettingsError {
        reason: e.to_string(),
    })?;

    Ok(())
}
