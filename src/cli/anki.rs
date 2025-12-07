use ulid::Ulid;

use crate::{
    application::ExportAnkiPackUseCase, cli::render_once, domain::JeersError,
    settings::ApplicationEnvironment,
};
use ratatui::{
    layout::Alignment,
    style::{Color, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

pub async fn handle_create_anki_pack(
    user_id: Ulid,
    file_path: String,
    word_tag: String,
    translation_tag: Option<String>,
    dry_run: bool,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();

    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(ratatui::style::Style::default().fg(Color::Yellow));
            let text = ratatui::text::Text::from(vec![Line::from(
                "Загрузка Anki файла...".fg(Color::Yellow),
            )]);
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .render(area, frame.buffer_mut());
        },
        5,
    )?;

    let use_case = ExportAnkiPackUseCase::new(
        settings.get_repository().await?,
        settings.get_embedding_service().await?,
        settings.get_llm_service().await?,
    );

    if dry_run {
        let cards = use_case
            .extract_cards(&file_path, &word_tag, translation_tag.as_deref())
            .await?;

        for anki_card in &cards {
            render_once(
                |frame| {
                    let area = frame.area();
                    let block = Block::bordered()
                        .border_set(border::ROUNDED)
                        .border_style(ratatui::style::Style::default().fg(Color::Yellow));
                    let text = ratatui::text::Text::from(vec![Line::from(
                        format!(
                            "Найдено слово: {}, перевод: {}",
                            anki_card.word,
                            anki_card.translation.as_deref().unwrap_or("None")
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
        }

        let text_lines = vec![
            Line::from("Dry run завершен".bold().fg(Color::Green)),
            Line::from(""),
            Line::from(format!("Найдено карточек: {}", cards.len())),
        ];

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

        return Ok(());
    }

    let result = use_case
        .execute(user_id, file_path, word_tag, translation_tag)
        .await?;

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
