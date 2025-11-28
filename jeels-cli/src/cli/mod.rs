pub mod anki;
mod card;
mod furigana_renderer;
mod learn;
mod migii;

use clap::Parser;
use ratatui::{
    Frame, Viewport,
    layout::{Alignment, Constraint, Layout, Rect},
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
    domain::{JapaneseLevel, JeersError, LessonHistoryItem, NativeLanguage, User},
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
        new_cards_force: bool,
        /// Show furigana by default
        #[clap(short, long, default_value = "false")]
        furigana_force: bool,
        /// Show similarity cards by default
        #[clap(short, long, default_value = "false")]
        similarity_force: bool,
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
        Command::Learn {
            new_cards_force,
            furigana_force,
            similarity_force,
        } => {
            handle_learn(user_id, new_cards_force, furigana_force, similarity_force).await?;
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
                Line::from(""),
                Line::from("Статистика карточек:".bold()),
                Line::from(format!("  Всего слов: {}", total_cards)),
                Line::from(format!("  Новых слов: {}", new_cards)),
                Line::from(format!("  Слов для изучения: {}", due_cards)),
            ];

            let lesson_history = user.lesson_history();

            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Magenta));

            let content_height = content.len() as u16;
            let graph_height = if !lesson_history.is_empty() { 10 } else { 0 };

            let vertical = Layout::vertical([
                Constraint::Length(content_height + 2),
                if graph_height > 0 {
                    Constraint::Length(graph_height)
                } else {
                    Constraint::Min(0)
                },
            ]);
            let [text_area, graph_area] = vertical.areas(content_area);

            Paragraph::new(Text::from(content))
                .block(block)
                .render(text_area, frame.buffer_mut());

            if !lesson_history.is_empty() && graph_height > 0 {
                draw_lesson_history_chart(frame, graph_area, lesson_history);
            }
        },
        if !user.lesson_history().is_empty() {
            40
        } else {
            25
        },
    )
    .map_err(|e| JeersError::SettingsError {
        reason: e.to_string(),
    })?;

    Ok(())
}

fn draw_lesson_history_chart(frame: &mut Frame, area: Rect, history: &[LessonHistoryItem]) {
    if history.is_empty() || area.width < 40 || area.height < 8 {
        return;
    }

    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [stability_area, difficulty_area] = horizontal.areas(area);

    draw_single_chart(
        frame,
        stability_area,
        history,
        "Стабильность",
        Color::Green,
        |item| item.avg_stability(),
    );
    draw_single_chart(
        frame,
        difficulty_area,
        history,
        "Сложность",
        Color::Red,
        |item| item.avg_difficulty(),
    );
}

fn draw_single_chart<F>(
    frame: &mut Frame,
    area: Rect,
    history: &[LessonHistoryItem],
    title: &str,
    color: Color,
    value_extractor: F,
) where
    F: Fn(&LessonHistoryItem) -> f64,
{
    if area.width < 20 || area.height < 8 {
        return;
    }

    let chart_block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(color))
        .title(title);

    let inner_area = chart_block.inner(area);
    chart_block.render(area, frame.buffer_mut());

    if inner_area.width < 15 || inner_area.height < 6 {
        return;
    }

    let label_width = 6;
    let chart_width = (inner_area.width.saturating_sub(label_width + 2)) as usize;
    let chart_height = inner_area.height.saturating_sub(3) as u16;
    let chart_start_x = inner_area.x + label_width;

    if chart_width == 0 || chart_height == 0 {
        return;
    }

    let values: Vec<f64> = history.iter().map(&value_extractor).collect();

    if values.is_empty() {
        return;
    }

    let min_value = values.iter().fold(f64::INFINITY, |a, &b| a.min(b)).max(0.0);
    let max_value = values
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        .max(0.1);

    let value_range = max_value - min_value;
    if value_range <= 0.0 {
        return;
    }

    fn value_to_y(value: f64, min: f64, max: f64, height: u16) -> u16 {
        if max == min {
            return height / 2;
        }
        let normalized = (value - min) / (max - min);
        (height as f64 * (1.0 - normalized)).round() as u16
    }

    let num_points = chart_width.min(history.len());
    let step = if history.len() > num_points {
        history.len() / num_points
    } else {
        1
    };

    let sampled_values: Vec<(usize, f64)> = history
        .iter()
        .enumerate()
        .step_by(step)
        .take(num_points)
        .map(|(i, item)| (i, value_extractor(item)))
        .collect();

    let buffer = frame.buffer_mut();
    let last_idx = sampled_values.len().saturating_sub(1);

    for (idx, &(_, value)) in sampled_values.iter().enumerate() {
        let x = idx.min(chart_width - 1);
        let cell_x = chart_start_x + x as u16;
        if cell_x >= inner_area.x + inner_area.width {
            break;
        }

        let value_y = value_to_y(value, min_value, max_value, chart_height);
        let line_y = inner_area.y + 1 + value_y;

        if idx > 0 {
            let prev_y = value_to_y(
                sampled_values[idx - 1].1,
                min_value,
                max_value,
                chart_height,
            );

            let prev_x = chart_start_x + (idx - 1) as u16;
            let dx = (cell_x - prev_x) as i32;
            let dy = value_y as i32 - prev_y as i32;

            for x_offset in 0..=dx {
                let x = prev_x + x_offset as u16;
                if x >= chart_start_x && x <= cell_x {
                    let progress = if dx > 0 {
                        x_offset as f64 / dx as f64
                    } else {
                        0.0
                    };

                    let y = (prev_y as f64 + dy as f64 * progress).round() as u16;
                    let draw_y = inner_area.y + 1 + y;

                    if draw_y < inner_area.y + inner_area.height
                        && x < inner_area.x + inner_area.width
                    {
                        let cell = &mut buffer[(x, draw_y)];
                        if cell.symbol() == " " {
                            cell.set_char('·');
                            cell.set_style(Style::default().fg(color));
                        }
                    }
                }
            }
        }

        if line_y < inner_area.y + inner_area.height {
            let cell = &mut buffer[(cell_x, line_y)];
            if cell.symbol() == "·" {
                cell.set_char('●');
                cell.set_style(Style::default().fg(color));
            } else {
                cell.set_char('●');
                cell.set_style(Style::default().fg(color));
            }

            if idx == last_idx {
                let label = format!("{:.1}", value);
                let label_start_x = cell_x + 1;
                for (i, ch) in label.chars().enumerate() {
                    let label_x = label_start_x + i as u16;
                    if label_x < inner_area.x + inner_area.width {
                        let label_cell = &mut buffer[(label_x, line_y)];
                        label_cell.set_char(ch);
                        label_cell.set_style(Style::default().fg(color));
                    }
                }
            }
        }
    }

    for y in 0..chart_height {
        let line_y = inner_area.y + 1 + y;
        if line_y >= inner_area.y + inner_area.height {
            continue;
        }

        let normalized_y = 1.0 - (y as f64 / (chart_height - 1) as f64);
        let value = min_value + (normalized_y * value_range);

        if y == 0 || y == chart_height - 1 || y == chart_height / 2 {
            let label = format!("{:>5.1}", value);
            for (i, ch) in label.chars().enumerate() {
                let label_x = inner_area.x + i as u16;
                if label_x < chart_start_x {
                    let cell = &mut buffer[(label_x, line_y)];
                    cell.set_char(ch);
                    cell.set_style(Style::default().fg(Color::Gray));
                }
            }

            let grid_x = chart_start_x;
            if grid_x < inner_area.x + inner_area.width {
                let cell = &mut buffer[(grid_x, line_y)];
                if cell.symbol() == " " {
                    cell.set_char('│');
                    cell.set_style(Style::default().fg(Color::DarkGray));
                }
            }
        }
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
