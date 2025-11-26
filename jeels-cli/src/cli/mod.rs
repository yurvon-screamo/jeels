pub mod anki;
mod card;
mod furigana_renderer;
mod learn;
mod migii;

use clap::Parser;
use ratatui::{
    Frame, Viewport,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use crate::{
    application::UserRepository,
    cli::{
        anki::handle_create_anki_pack,
        card::{
            handle_create_card, handle_create_words, handle_delete_card, handle_edit_card,
            handle_list_cards, handle_rebuild_database,
        },
        learn::handle_learn,
        migii::handle_create_migii_pack,
    },
    domain::{JapaneseLevel, JeersError, NativeLanguage, User},
    settings::ApplicationEnvironment,
};

const DEFAULT_USERNAME: &str = "yurvon_screamo";
const DEFAULT_JAPANESE_LEVEL: JapaneseLevel = JapaneseLevel::N5;
const DEFAULT_NATIVE_LANGUAGE: NativeLanguage = NativeLanguage::Russian;
const DEFAULT_NEW_CARDS_LIMIT: usize = 15;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = DEFAULT_USERNAME)]
    username: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Show user information
    Me {},
    /// Learn cards
    Learn {
        /// Ignore new cards limit and show all due cards
        #[clap(short, long, default_value = "false")]
        force_new_cards: bool,
    },
    /// List cards
    Cards {},
    /// Create card
    Create {
        // Question to create
        question: String,
        // Answer to create
        answer: String,
    },
    // Bulk create cards
    CreateWords {
        // Questions to create (answer will be generated)
        questions: Vec<String>,
    },
    // Edit card
    Edit {
        // Card ID to edit
        card_id: Ulid,
        // New question
        question: String,
        // New answer
        answer: String,
    },
    // Delete cards
    Delete {
        // Card IDs to delete
        card_ids: Vec<Ulid>,
    },
    // Import Migii vocabulary lessons
    MigiiCreate {
        // Lessons numbers to import
        lessons: Vec<u32>,
        // If true, only questions will be imported, answers will be generated
        #[clap(short, long, default_value = "false")]
        question_only: bool,
    },
    // Import Anki vocabulary from file
    AnkiCreate {
        // File path to Anki desk file
        file_path: String,
        // Tag for word field
        word_tag: String,
        // Tag for translation field (if not provided, translation will be generated)
        translation_tag: Option<String>,
        // If true, words will printed, but not saved
        #[clap(short, long, default_value = "false")]
        dry_run: bool,
    },
    // Rebuild embedding and answers for all cards
    RebuildDatabase {
        // Only embedding will be rebuilt, answers will not be regenerated
        #[clap(short, long, default_value = "false")]
        embedding_only: bool,
    },
}

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let user_id = ensure_user_exists(ApplicationEnvironment::get(), &args.username).await?;

    match args.command {
        Command::Me {} => {
            handle_me(user_id).await?;
        }
        Command::Cards {} => {
            handle_list_cards(user_id).await?;
        }
        Command::Learn { force_new_cards } => {
            handle_learn(user_id, force_new_cards).await?;
        }
        Command::Create { question, answer } => {
            handle_create_card(user_id, question, answer).await?;
        }
        Command::CreateWords { questions } => {
            handle_create_words(user_id, questions).await?;
        }
        Command::Edit {
            card_id,
            question,
            answer,
        } => {
            handle_edit_card(user_id, card_id, question, answer).await?;
        }
        Command::Delete { card_ids } => {
            handle_delete_card(user_id, card_ids).await?;
        }
        Command::MigiiCreate {
            lessons,
            question_only,
        } => {
            handle_create_migii_pack(user_id, lessons, question_only).await?;
        }
        Command::AnkiCreate {
            file_path,
            word_tag,
            translation_tag,
            dry_run,
        } => {
            handle_create_anki_pack(user_id, file_path, word_tag, translation_tag, dry_run).await?;
        }
        Command::RebuildDatabase { embedding_only } => {
            handle_rebuild_database(user_id, embedding_only).await?;
        }
    }

    Ok(())
}

async fn ensure_user_exists(
    settings: &'static ApplicationEnvironment,
    username: &str,
) -> Result<Ulid, Box<dyn std::error::Error>> {
    let repository = settings.get_repository().await?;

    if let Some(user) = repository
        .find_by_username(username)
        .await
        .map_err(|e| format!("Failed to find user: {}", e))?
    {
        Ok(user.id())
    } else {
        let new_user = User::new(
            username.to_string(),
            DEFAULT_JAPANESE_LEVEL,
            DEFAULT_NATIVE_LANGUAGE,
            DEFAULT_NEW_CARDS_LIMIT,
        );
        let user_id = new_user.id();
        repository
            .save(&new_user)
            .await
            .map_err(|e| format!("Failed to save user: {}", e))?;
        Ok(user_id)
    }
}

async fn handle_me(user_id: Ulid) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;
    let user = repository
        .find_by_id(user_id)
        .await?
        .ok_or(JeersError::UserNotFound { user_id })?;

    let cards = user.cards();
    let total_cards = cards.len();
    let new_cards = cards.values().filter(|card| card.is_new()).count();
    let due_cards = cards
        .values()
        .filter(|card| card.is_due() && !card.is_new())
        .count();

    let stabilities: Vec<f64> = cards
        .values()
        .filter_map(|card| card.stability().map(|s| s.value()))
        .collect();
    let difficulties: Vec<f64> = cards
        .values()
        .filter_map(|card| card.difficulty().map(|d| d.value()))
        .collect();

    let stability_stats = if stabilities.is_empty() {
        None
    } else {
        let avg = stabilities.iter().sum::<f64>() / stabilities.len() as f64;
        let min = stabilities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = stabilities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        Some((avg, min, max))
    };

    let difficulty_stats = if difficulties.is_empty() {
        None
    } else {
        let avg = difficulties.iter().sum::<f64>() / difficulties.len() as f64;
        let min = difficulties.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = difficulties
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        Some((avg, min, max))
    };

    render_once(
        |frame| {
            let area = frame.area();
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [title_area, content_area] = vertical.areas(area);

            let title = Line::from("Информация о пользователе:".bold().underlined());
            Paragraph::new(title)
                .alignment(Alignment::Left)
                .render(title_area, frame.buffer_mut());

            let mut content = vec![
                Line::from(format!("ID: {}", user.id())),
                Line::from(format!("Имя пользователя: {}", user.username())),
                Line::from(format!(
                    "Уровень японского: {:?}",
                    user.current_japanese_level()
                )),
                Line::from(format!("Родной язык: {:?}", user.native_language())),
                Line::from(""),
                Line::from("Статистика карточек:".bold()),
                Line::from(format!("  Всего слов: {}", total_cards)),
                Line::from(format!("  Новых слов: {}", new_cards)),
                Line::from(format!("  Слов для изучения: {}", due_cards)),
            ];

            if let Some((avg, min, max)) = stability_stats {
                content.push(Line::from(""));
                content.push(Line::from("Стабильность:".bold()));
                content.push(Line::from(format!("  Среднее: {:.2}", avg)));
                content.push(Line::from(format!("  Минимум: {:.2}", min)));
                content.push(Line::from(format!("  Максимум: {:.2}", max)));
            } else {
                content.push(Line::from(""));
                content.push(Line::from("Стабильность: нет данных"));
            }

            if let Some((avg, min, max)) = difficulty_stats {
                content.push(Line::from(""));
                content.push(Line::from("Сложность:".bold()));
                content.push(Line::from(format!("  Среднее: {:.2}", avg)));
                content.push(Line::from(format!("  Минимум: {:.2}", min)));
                content.push(Line::from(format!("  Максимум: {:.2}", max)));
            } else {
                content.push(Line::from(""));
                content.push(Line::from("Сложность: нет данных"));
            }

            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Magenta));

            Paragraph::new(Text::from(content))
                .block(block)
                .render(content_area, frame.buffer_mut());
        },
        25,
    )
    .map_err(|e| JeersError::SettingsError {
        reason: e.to_string(),
    })?;

    Ok(())
}

pub(crate) fn render_once<F>(draw_fn: F, lines: u16) -> Result<(), JeersError>
where
    F: FnOnce(&mut Frame),
{
    let mut terminal = ratatui::init_with_options(ratatui::TerminalOptions {
        viewport: Viewport::Inline(lines),
    });

    terminal
        .draw(draw_fn)
        .map_err(|e| JeersError::SettingsError {
            reason: e.to_string(),
        })?;
    Ok(())
}
