mod card;
mod learn;

use clap::Parser;
use iocraft::prelude::*;
use ulid::Ulid;

use crate::{
    application::UserRepository,
    cli::{
        card::{
            handle_create_card, handle_create_words, handle_delete_card, handle_edit_card,
            handle_list_cards,
        },
        learn::handle_learn,
    },
    domain::{JapaneseLevel, JeersError, NativeLanguage, User},
    settings::Settings,
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
}

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let user_id = ensure_user_exists(Settings::get(), &args.username).await?;

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
    }

    Ok(())
}

async fn ensure_user_exists(
    settings: &'static Settings,
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
    let settings = Settings::get();
    let repository = settings.get_repository().await?;
    let user = repository
        .find_by_id(user_id)
        .await?
        .ok_or(JeersError::UserNotFound { user_id })?;
    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: 1,
            margin_bottom: 1,
        ) {
            Text(content: "Информация о пользователе:", weight: Weight::Bold, decoration: TextDecoration::Underline)
            View(
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color: Color::Magenta,
            ) {
                Text(content: format!("ID: {}", user.id()))
                Text(content: format!("Имя пользователя: {}", user.username()))
                Text(content: format!("Уровень японского: {:?}", user.current_japanese_level()))
                Text(content: format!("Родной язык: {:?}", user.native_language()))
            }
        }
    }
    .print();
    Ok(())
}
