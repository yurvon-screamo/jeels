use regex::Regex;
use rusqlite::Connection;
use serde_json::Value;
use ulid::Ulid;

use crate::{
    application::{CreateCardUseCase, UserRepository},
    cli::render_once,
    domain::JeersError,
    settings::ApplicationEnvironment,
};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::fs::File;
use std::io::Cursor;
use zip::ZipArchive;

#[derive(Debug)]
pub struct AnkiCard {
    pub word: String,
    pub translation: Option<String>,
}

pub async fn handle_create_anki_pack(
    user_id: Ulid,
    file_path: String,
    word_tag: String,
    translation_tag: Option<String>,
    dry_run: bool,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;

    repository
        .find_by_id(user_id)
        .await?
        .ok_or(JeersError::UserNotFound { user_id })?;

    let use_case = CreateCardUseCase::new(
        settings.get_repository().await?,
        settings.get_embedding_generator().await?,
        settings.get_llm_service().await?,
    );

    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Yellow));
            let text = Text::from(vec![Line::from("Загрузка Anki файла...".fg(Color::Yellow))]);
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .render(area, frame.buffer_mut());
        },
        5,
    )?;

    let bytes = tokio::fs::read(file_path)
        .await
        .map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to read file: {}", e),
        })?;

    let cards =
        extract_anki_cards(&bytes[..], &word_tag, translation_tag.as_deref()).map_err(|e| {
            JeersError::RepositoryError {
                reason: format!("Failed to extract Anki cards: {}", e),
            }
        })?;

    let total_words = cards.len();
    let mut created_count = 0;
    let mut total_skipped_words = Vec::new();

    for anki_card in cards {
        let question = anki_card.word;
        let answer = anki_card.translation;

        if dry_run {
            created_count += 1;
            render_once(
                |frame| {
                    let area = frame.area();
                    let block = Block::bordered()
                        .border_set(border::ROUNDED)
                        .border_style(Style::default().fg(Color::Yellow));
                    let text = Text::from(vec![Line::from(
                        format!(
                            "Найдено слово: {}, перевод: {}",
                            question,
                            answer.unwrap_or("None".to_string())
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
        } else {
            match use_case.execute(user_id, question.clone(), answer).await {
                Ok(card) => {
                    created_count += 1;
                    render_once(
                        |frame| {
                            let area = frame.area();
                            let vertical =
                                Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
                            let [title_area, card_area] = vertical.areas(area);

                            let title = Line::from(
                                format!("Создана карточка {}/{}:", created_count, total_words)
                                    .bold()
                                    .underlined(),
                            );
                            Paragraph::new(title)
                                .alignment(Alignment::Left)
                                .render(title_area, frame.buffer_mut());

                            let card_block = Block::bordered()
                                .border_set(border::ROUNDED)
                                .border_style(Style::default().fg(Color::Green));
                            let card_text = Text::from(vec![
                                Line::from(format!("Вопрос: {}", card.question().text())),
                                Line::from(format!("Ответ: {}", card.answer().text())),
                            ]);
                            Paragraph::new(card_text)
                                .block(card_block)
                                .render(card_area, frame.buffer_mut());
                        },
                        8,
                    )?;
                }
                Err(JeersError::DuplicateCard { .. }) => {
                    total_skipped_words.push(question);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    let mut text_lines = vec![
        Line::from("Пачка создана успешно!".bold().fg(Color::Green)),
        Line::from(""),
        Line::from(format!("Создано карточек: {}", created_count)),
        Line::from(format!(
            "Пропущено (дубликаты): {}",
            total_skipped_words.len()
        )),
    ];

    if !total_skipped_words.is_empty() {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from("Пропущенные слова:".bold().fg(Color::Yellow)));
        for word in &total_skipped_words {
            text_lines.push(Line::from(format!("  • {}", word).fg(Color::Gray)));
        }
    }

    let height = (text_lines.len() + 2) as u16;
    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Green));
            Paragraph::new(Text::from(text_lines))
                .block(block)
                .alignment(Alignment::Left)
                .render(area, frame.buffer_mut());
        },
        height,
    )?;

    Ok(())
}

pub fn extract_anki_cards(
    data: &[u8],
    word_tag: &str,
    translation_tag: Option<&str>,
) -> Result<Vec<AnkiCard>, Box<dyn std::error::Error>> {
    // Open ZIP archive
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    // Extract database file
    let mut db_file_entry = archive.by_name("collection.anki21")?;

    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("collection.anki21");
    let mut temp_db_file = File::create(&db_path)?;

    std::io::copy(&mut db_file_entry, &mut temp_db_file)?;

    // Connect to SQLite
    let conn = Connection::open(&db_path)?;

    // Get models JSON from col table
    let mut stmt = conn.prepare("SELECT models FROM col")?;
    let json_str: String = stmt.query_row([], |row| row.get(0))?;

    // Parse JSON to find field indices
    let models: Value = serde_json::from_str(&json_str)?;

    // Find field indices by name
    let mut word_index = None;
    let mut translation_index = None;

    if let Some(models_map) = models.as_object() {
        // Iterate through all models to find the one with matching fields
        for (_model_id, model_data) in models_map {
            if let Some(fields) = model_data["flds"].as_array() {
                for (index, field) in fields.iter().enumerate() {
                    if let Some(field_name) = field["name"].as_str() {
                        if field_name.to_lowercase() == word_tag.to_lowercase() {
                            word_index = Some(index);
                        }
                        if let Some(trans_tag) = translation_tag {
                            if field_name.to_lowercase() == trans_tag.to_lowercase() {
                                translation_index = Some(index);
                            }
                        }
                    }
                }

                // If we found both indices, we can break
                if word_index.is_some()
                    && (translation_tag.is_none() || translation_index.is_some())
                {
                    break;
                }
            }
        }
    }

    let word_index =
        word_index.ok_or_else(|| format!("Field '{}' not found in Anki deck models", word_tag))?;

    // Regex for HTML cleanup
    let re_html = Regex::new(r"<[^>]*>")?;
    let re_nbsp = Regex::new(r"&nbsp;")?;

    // Read notes from database
    let mut stmt = conn.prepare("SELECT flds FROM notes")?;
    let rows = stmt.query_map([], |row| {
        let flds: String = row.get(0)?;
        Ok(flds)
    })?;

    let mut cards = Vec::new();

    let clean_text = |raw: &str| -> String {
        let no_html = re_html.replace_all(raw, " ");
        let no_nbsp = re_nbsp.replace_all(&no_html, " ");
        no_nbsp.trim().to_string()
    };

    for row in rows {
        let flds_str = row?;
        let fields: Vec<&str> = flds_str.split('\x1f').collect();

        let raw_word = fields.get(word_index).unwrap_or(&"");
        let word = clean_text(raw_word);

        let translation = if let Some(translation_index) = translation_index {
            let raw_translation = fields.get(translation_index).unwrap_or(&"");
            Some(clean_text(raw_translation))
        } else {
            None
        };

        if !word.is_empty() {
            cards.push(AnkiCard { word, translation });
        }
    }

    Ok(cards)
}
