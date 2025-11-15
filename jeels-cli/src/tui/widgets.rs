use crate::domain::Card;
use crate::tui::ui::theme::NeonTheme;
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Wrap};

pub fn create_card_widget(card: &Card, show_answer: bool) -> Paragraph<'_> {
    let question_text = card.question().text();
    let answer_text = card.answer().text();

    if show_answer {
        let line = Line::from(vec![
            Span::styled(
                format!("{} ", NeonTheme::SPARKLE),
                Style::default()
                    .fg(NeonTheme::PURPLE_NEON)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                question_text,
                Style::default()
                    .fg(NeonTheme::PURPLE_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" → ", Style::default().fg(NeonTheme::TEXT_DIM)),
            Span::styled(
                format!("{} ", NeonTheme::STAR),
                Style::default()
                    .fg(NeonTheme::GREEN_NEON)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                answer_text,
                Style::default()
                    .fg(NeonTheme::GREEN_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        Paragraph::new(line).wrap(Wrap { trim: true })
    } else {
        let line = Line::from(vec![
            Span::styled(
                format!("{} ", NeonTheme::SPARKLE),
                Style::default()
                    .fg(NeonTheme::PURPLE_NEON)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                question_text,
                Style::default()
                    .fg(NeonTheme::PURPLE_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        Paragraph::new(line).wrap(Wrap { trim: true })
    }
}

pub fn create_empty_card_widget() -> Paragraph<'static> {
    let line = Line::from(vec![
        Span::styled(
            format!("{} ", NeonTheme::SPARKLES),
            Style::default()
                .fg(NeonTheme::GREEN_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Все карточки изучены!",
            Style::default()
                .fg(NeonTheme::GREEN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    Paragraph::new(line).wrap(Wrap { trim: true })
}

pub fn get_instructions_text(show_answer: bool) -> Vec<Line<'static>> {
    if show_answer {
        vec![
            Line::from(vec![
                Span::styled("1: ", Style::default().fg(NeonTheme::MAGENTA)),
                Span::styled(
                    "❌ ",
                    Style::default()
                        .fg(NeonTheme::MAGENTA)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Снова | ", Style::default().fg(NeonTheme::TEXT_DIM)),
                Span::styled("2: ", Style::default().fg(NeonTheme::YELLOW)),
                Span::styled(
                    "🟠 ",
                    Style::default()
                        .fg(NeonTheme::YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Сложно | ", Style::default().fg(NeonTheme::TEXT_DIM)),
                Span::styled("3: ", Style::default().fg(NeonTheme::GREEN_NEON)),
                Span::styled(
                    "🟢 ",
                    Style::default()
                        .fg(NeonTheme::GREEN_NEON)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Хорошо | ", Style::default().fg(NeonTheme::TEXT_DIM)),
                Span::styled("4: ", Style::default().fg(NeonTheme::CYAN)),
                Span::styled(
                    "⭐ ",
                    Style::default()
                        .fg(NeonTheme::CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Легко", Style::default().fg(NeonTheme::TEXT_DIM)),
            ]),
            Line::from(vec![
                Span::styled(
                    "N/→: ",
                    Style::default()
                        .fg(NeonTheme::GREEN_NEON)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Следующая | ", Style::default().fg(NeonTheme::TEXT_DIM)),
                Span::styled(
                    "Q/ESC: ",
                    Style::default()
                        .fg(NeonTheme::MAGENTA)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Выход", Style::default().fg(NeonTheme::TEXT_DIM)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled(
                    "SPACE: ",
                    Style::default()
                        .fg(NeonTheme::PURPLE_BRIGHT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Показать ответ", Style::default().fg(NeonTheme::TEXT_DIM)),
            ]),
            Line::from(vec![
                Span::styled(
                    "N/→: ",
                    Style::default()
                        .fg(NeonTheme::GREEN_NEON)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Следующая | ", Style::default().fg(NeonTheme::TEXT_DIM)),
                Span::styled(
                    "Q/ESC: ",
                    Style::default()
                        .fg(NeonTheme::MAGENTA)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Выход", Style::default().fg(NeonTheme::TEXT_DIM)),
            ]),
        ]
    }
}
