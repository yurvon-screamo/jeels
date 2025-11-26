use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
struct MigiiResponse {
    data: Vec<MigiiWord>,
}

#[derive(Debug, Deserialize)]
struct MigiiWord {
    word: String,
    short_mean: String,
    mean: Vec<MigiiMeaning>,
}

#[derive(Debug, Deserialize)]
struct MigiiMeaning {
    mean: String,
}

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
    let level_num = level.as_number();
    let native_lang = match user.native_language() {
        crate::domain::NativeLanguage::Russian => "ru",
        crate::domain::NativeLanguage::English => "en",
    };

    let use_case = CreateCardUseCase::new(
        settings.get_repository().await?,
        settings.get_embedding_generator().await?,
        settings.get_llm_service().await?,
    );

    let mut total_created_count = 0;
    let mut total_skipped_words = Vec::new();

    for lesson in lessons {
        let url = format!(
            "https://v2.migii.net/api/theory/word/javi/{}/{}/{}",
            native_lang, level_num, lesson
        );

        render_once(
            |frame| {
                let area = frame.area();
                let block = Block::bordered()
                    .border_set(border::ROUNDED)
                    .border_style(Style::default().fg(Color::Yellow));
                let text = Text::from(vec![Line::from(
                    format!("Загрузка урока {} уровня {:?}...", lesson, level).fg(Color::Yellow),
                )]);
                Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Center)
                    .render(area, frame.buffer_mut());
            },
            5,
        )?;

        let response = reqwest::get(&url)
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to fetch Migii data: {}", e),
            })?;

        let migii_data: MigiiResponse =
            response
                .json()
                .await
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to parse Migii JSON: {}", e),
                })?;

        let total_words = migii_data.data.len();
        let mut created_count = 0;

        for word_data in migii_data.data {
            let question = word_data.word.clone();
            let answer = if !word_data.short_mean.is_empty() {
                word_data.short_mean
            } else if let Some(first_meaning) = word_data.mean.first() {
                first_meaning.mean.clone()
            } else {
                continue;
            };
            let answer = if question_only { None } else { Some(answer) };

            match use_case.execute(user_id, question.clone(), answer).await {
                Ok(card) => {
                    created_count += 1;
                    total_created_count += 1;
                    render_once(
                        |frame| {
                            let area = frame.area();
                            let vertical =
                                Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
                            let [title_area, card_area] = vertical.areas(area);

                            let title = Line::from(
                                format!(
                                    "Создана карточка {}/{} (урок {}):",
                                    created_count, total_words, lesson
                                )
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
        Line::from(format!("Создано карточек: {}", total_created_count)),
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
