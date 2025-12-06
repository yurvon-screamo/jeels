pub mod anki;
mod card;
mod duolingo;
mod furigana_renderer;
mod kanji;
mod learn;
mod me;
mod migii;
mod translate;

use clap::Parser;
use ratatui::{Frame, Viewport};
use ulid::Ulid;

use crate::{
    application::UserRepository,
    cli::{
        anki::handle_create_anki_pack,
        card::{
            handle_create_card, handle_create_words, handle_delete_card, handle_edit_card,
            handle_list_cards, handle_rebuild_database,
        },
        duolingo::handle_sync_duolingo_words,
        kanji::handle_kanji,
        learn::handle_learn,
        me::handle_me,
        migii::handle_create_migii_pack,
        translate::handle_translate,
    },
    domain::{
        JeersError, User,
        value_objects::{JapaneseLevel, NativeLanguage},
    },
    settings::ApplicationEnvironment,
};

const DEFAULT_USERNAME: &str = "yurvon_screamo";
const DEFAULT_JAPANESE_LEVEL: JapaneseLevel = JapaneseLevel::N5;
const DEFAULT_NATIVE_LANGUAGE: NativeLanguage = NativeLanguage::Russian;
const DEFAULT_NEW_CARDS_LIMIT: usize = 7;

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
        new_cards_force: bool,
        /// Show furigana by default
        #[clap(short, long, default_value = "false")]
        furigana_force: bool,
        /// Show similarity cards by default
        #[clap(short, long, default_value = "false")]
        similarity_force: bool,
        /// Loop mode: continuously cycle through low stability cards
        #[clap(short, long, default_value = "false")]
        loop_mod: bool,
    },
    /// List cards
    Cards {},
    /// Create card
    Create {
        /// Question to create
        question: String,
        /// Answer to create
        answer: String,
    },
    /// Bulk create cards
    CreateWords {
        /// Questions to create (answer will be generated)
        questions: Vec<String>,
    },
    /// Edit card
    Edit {
        /// Card ID to edit
        card_id: Ulid,
        /// New question
        question: String,
        /// New answer
        answer: String,
    },
    /// Delete cards
    Delete {
        /// Card IDs to delete
        card_ids: Vec<Ulid>,
    },
    /// Import Migii vocabulary lessons
    MigiiCreate {
        /// Lessons numbers to import
        lessons: Vec<u32>,
        /// If true, only questions will be imported, answers will be generated
        #[clap(short, long, default_value = "false")]
        question_only: bool,
    },
    /// Import Anki vocabulary from file
    AnkiCreate {
        /// File path to Anki desk file
        file_path: String,
        /// Tag for word field
        word_tag: String,
        /// Tag for translation field (if not provided, translation will be generated)
        translation_tag: Option<String>,
        /// If true, words will printed, but not saved
        #[clap(short, long, default_value = "false")]
        dry_run: bool,
    },
    /// Rebuild embedding and answers for all cards
    RebuildDatabase {
        /// If true, example phrases will be rebuilt
        #[clap(long, default_value = "false")]
        rebuild_example_phrases: bool,
        /// If true, embedding will be rebuilt
        #[clap(long, default_value = "false")]
        rebuild_embedding: bool,
        /// If true, answers will be rebuilt
        #[clap(long, default_value = "false")]
        rebuild_answer: bool,
    },
    /// Translate text (auto-detects Japanese or native language)
    Translate {
        /// Text to translate
        text: String,
    },
    /// Get information about a kanji character
    Kanji {
        /// Kanji character to get information about
        kanji: String,
    },
    /// Sync words from Duolingo
    DuolingoSync {
        /// If true, only questions will be imported, answers will be generated
        #[clap(short, long, default_value = "false")]
        question_only: bool,
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
        Command::Learn {
            new_cards_force,
            furigana_force,
            similarity_force,
            loop_mod,
        } => {
            handle_learn(
                user_id,
                new_cards_force,
                furigana_force,
                similarity_force,
                loop_mod,
            )
            .await?;
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
        Command::RebuildDatabase {
            rebuild_example_phrases,
            rebuild_embedding,
            rebuild_answer,
        } => {
            handle_rebuild_database(
                user_id,
                rebuild_example_phrases,
                rebuild_embedding,
                rebuild_answer,
            )
            .await?;
        }
        Command::Translate { text } => {
            handle_translate(user_id, text).await?;
        }
        Command::Kanji { kanji } => {
            handle_kanji(user_id, kanji).await?;
        }
        Command::DuolingoSync { question_only } => {
            handle_sync_duolingo_words(user_id, question_only).await?;
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
