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

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = DEFAULT_USERNAME)]
    username: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Me {},
    Learn {},
    Cards {},
    Create {
        question: String,
        answer: String,
    },
    CreateWords {
        questions: Vec<String>,
    },
    Edit {
        card_id: Ulid,
        question: String,
        answer: String,
    },
    Delete {
        card_ids: Vec<Ulid>,
    },
    MigiiCreate {
        lesson: u32,
    },
    RebuildDatabase {},
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
        Command::Learn {} => {
            handle_learn(user_id).await?;
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
        Command::MigiiCreate { lesson } => {
            handle_create_migii_pack(user_id, lesson).await?;
        }
        Command::RebuildDatabase {} => {
            handle_rebuild_database(user_id).await?;
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

    render_once(
        |frame| {
            let area = frame.area();
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [title_area, content_area] = vertical.areas(area);

            let title = Line::from("Информация о пользователе:".bold().underlined());
            Paragraph::new(title)
                .alignment(Alignment::Left)
                .render(title_area, frame.buffer_mut());

            let content = vec![
                Line::from(format!("ID: {}", user.id())),
                Line::from(format!("Имя пользователя: {}", user.username())),
                Line::from(format!(
                    "Уровень японского: {:?}",
                    user.current_japanese_level()
                )),
                Line::from(format!("Родной язык: {:?}", user.native_language())),
            ];

            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Magenta));

            Paragraph::new(Text::from(content))
                .block(block)
                .render(content_area, frame.buffer_mut());
        },
        7,
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
