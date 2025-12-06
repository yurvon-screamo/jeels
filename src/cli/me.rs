use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use crate::{
    application::use_cases::GetUserInfoUseCase,
    domain::{JeersError, daily_history::DailyHistoryItem},
    settings::ApplicationEnvironment,
};

use super::render_once;

pub async fn handle_me(user_id: Ulid) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await?;
    let use_case = GetUserInfoUseCase::new(repository);
    let user = use_case.execute(user_id).await?;

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
                Line::from(format!("ID: {}", user.id)),
                Line::from(format!("Имя пользователя: {}", user.username)),
                Line::from(format!(
                    "Уровень японского: {:?}",
                    user.current_japanese_level
                )),
                Line::from(format!("Родной язык: {:?}", user.native_language)),
            ];

            let lesson_history = &user.lesson_history;

            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Magenta));

            let content_height = content.len() as u16;
            let graph_height = if !lesson_history.is_empty() { 20 } else { 0 };

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
        if !user.lesson_history.is_empty() {
            50
        } else {
            15
        },
    )
    .map_err(|e| JeersError::SettingsError {
        reason: e.to_string(),
    })?;

    Ok(())
}

fn draw_lesson_history_chart(frame: &mut Frame, area: Rect, history: &[DailyHistoryItem]) {
    if history.is_empty() || area.width < 40 || area.height < 16 {
        return;
    }

    let vertical = Layout::vertical([
        Constraint::Length(area.height / 2),
        Constraint::Length(area.height / 2),
    ]);
    let [top_row, bottom_row] = vertical.areas(area);

    let top_horizontal = Layout::horizontal([
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(34),
    ]);
    let [stability_area, difficulty_area, total_words_area] = top_horizontal.areas(top_row);

    let bottom_horizontal =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [known_words_area, new_words_area] = bottom_horizontal.areas(bottom_row);

    draw_single_chart(
        frame,
        stability_area,
        history,
        "Стабильность",
        Color::Green,
        |item| item.avg_stability(),
        false,
    );
    draw_single_chart(
        frame,
        difficulty_area,
        history,
        "Сложность",
        Color::Red,
        |item| item.avg_difficulty(),
        false,
    );
    draw_single_chart(
        frame,
        total_words_area,
        history,
        "Всего слов",
        Color::Blue,
        |item| item.total_words() as f64,
        true,
    );
    draw_single_chart(
        frame,
        known_words_area,
        history,
        "Изученных",
        Color::Yellow,
        |item| item.known_words() as f64,
        true,
    );
    draw_single_chart(
        frame,
        new_words_area,
        history,
        "Новых",
        Color::Cyan,
        |item| item.new_words() as f64,
        true,
    );
}

fn draw_single_chart<F>(
    frame: &mut Frame,
    area: Rect,
    history: &[DailyHistoryItem],
    title: &str,
    color: Color,
    value_extractor: F,
    is_integer: bool,
) where
    F: Fn(&DailyHistoryItem) -> f64,
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
    let chart_height = inner_area.height.saturating_sub(3);
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
    let is_constant = value_range <= 0.0;

    fn value_to_y(value: f64, min: f64, max: f64, height: u16, is_constant: bool) -> u16 {
        if is_constant || max == min {
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

        let value_y = value_to_y(value, min_value, max_value, chart_height, is_constant);
        let line_y = inner_area.y + 1 + value_y;

        if idx > 0 {
            let prev_y = value_to_y(
                sampled_values[idx - 1].1,
                min_value,
                max_value,
                chart_height,
                is_constant,
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
                let label = if is_integer {
                    format!("{:.0}", value)
                } else {
                    format!("{:.1}", value)
                };
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
        let value = if is_constant {
            min_value
        } else {
            min_value + (normalized_y * value_range)
        };

        if y == 0 || y == chart_height - 1 || y == chart_height / 2 {
            let label = if is_integer {
                format!("{:>5.0}", value)
            } else if is_constant && y == chart_height / 2 {
                format!("{:>5.0}", value)
            } else {
                format!("{:>5.1}", value)
            };
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
