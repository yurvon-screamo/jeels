use crate::tui::app::AppState;
use crate::tui::ui::common::{create_vertical_layout, render_footer};
use crate::tui::ui::theme::NeonTheme;
use chrono::Utc;
use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, Paragraph};

pub fn render(frame: &mut Frame, app_state: &AppState) {
    let layout = create_vertical_layout(
        frame.area(),
        &[
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ],
    );

    render_header(frame, &layout[0], app_state);
    render_list(frame, &layout[1], app_state);
    render_footer(
        frame,
        &layout[2],
        "↑↓: Навигация | Enter: Просмотр | E: Редактировать | D: Удалить | ESC: Назад",
        NeonTheme::GREEN_NEON,
    );
}

fn render_header(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", NeonTheme::DIAMOND),
            Style::default()
                .fg(NeonTheme::GREEN_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Карточки",
            Style::default()
                .fg(NeonTheme::GREEN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({})", app_state.all_cards.len()),
            Style::default().fg(NeonTheme::PURPLE_BRIGHT),
        ),
    ]));
    frame.render_widget(header, *area);
}

fn render_list(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .all_cards
        .iter()
        .enumerate()
        .map(|(idx, card)| {
            let is_selected = idx == app_state.card_list_index;
            let question_preview = if card.question().text().len() > 40 {
                format!("{}...", &card.question().text()[..40])
            } else {
                card.question().text().to_string()
            };

            let (status_icon, status_text, status_color) = calculate_card_status(card);

            let arrow = if is_selected {
                format!("{} ", NeonTheme::SPARKLE)
            } else {
                "  ".to_string()
            };

            let title_style = if is_selected {
                Style::default()
                    .fg(NeonTheme::PURPLE_BRIGHT)
                    .bg(NeonTheme::BG_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
                    .fg(NeonTheme::PURPLE_BRIGHT)
                    .add_modifier(Modifier::BOLD)
            };

            let status_style = if is_selected {
                Style::default()
                    .fg(status_color)
                    .bg(NeonTheme::BG_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(arrow, Style::default().fg(NeonTheme::GREEN_NEON)),
                    Span::styled(
                        format!("{} {}", NeonTheme::DIAMOND, question_preview),
                        title_style,
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("   {} {}", status_icon, status_text),
                        status_style,
                    ),
                    Span::styled(
                        format!(
                            " | {} Повторений: {} | {} Стабильность: {:.2}",
                            NeonTheme::SPARKLE,
                            card.reviews().len(),
                            NeonTheme::STAR,
                            card.stability().value()
                        ),
                        Style::default().fg(if is_selected {
                            NeonTheme::TEXT_DIM
                        } else {
                            NeonTheme::TEXT_DARK
                        }),
                    ),
                ]),
            ])
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, *area);
}

fn calculate_card_status(card: &crate::domain::Card) -> (&'static str, String, Color) {
    let is_due = card.is_due();
    let next_review = card.next_review_date();
    let now = Utc::now();
    let days_until_review = if next_review > now {
        let duration = next_review - now;
        duration.num_days()
    } else {
        0
    };

    if is_due {
        (
            "🔴",
            "Готова к повторению".to_string(),
            NeonTheme::MAGENTA,
        )
    } else if days_until_review == 0 {
        (
            "🟡",
            "Сегодня".to_string(),
            NeonTheme::YELLOW,
        )
    } else if days_until_review == 1 {
        (
            "🟡",
            "Завтра".to_string(),
            NeonTheme::YELLOW,
        )
    } else {
        (
            "🟢",
            format!("Через {} дн.", days_until_review),
            NeonTheme::GREEN_NEON,
        )
    }
}
