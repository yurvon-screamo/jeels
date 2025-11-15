use crate::tui::app::{AppState, InputField, Screen};
use crate::tui::ui::common::{create_vertical_layout, render_footer};
use crate::tui::ui::theme::NeonTheme;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

pub fn render(frame: &mut Frame, app_state: &AppState) {
    if let Screen::CardEdit {
        question, answer, ..
    } = &app_state.screen
    {
        let layout = create_vertical_layout(
            frame.area(),
            &[
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        );

        render_header(frame, &layout[0]);
        render_question_field(frame, &layout[1], question, app_state);
        render_answer_field(frame, &layout[2], answer, app_state);
        render_footer(
            frame,
            &layout[3],
            "↑↓/Tab: Переключение | Enter: Сохранить | Backspace: Удалить | ESC: Отмена",
            NeonTheme::PURPLE_NEON,
        );
    }
}

fn render_header(frame: &mut Frame, area: &Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", NeonTheme::SPARKLE),
            Style::default()
                .fg(NeonTheme::PURPLE_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Редактирование",
            Style::default()
                .fg(NeonTheme::PURPLE_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(header, *area);
}

fn render_question_field(
    frame: &mut Frame,
    area: &Rect,
    question: &str,
    app_state: &AppState,
) {
    let is_active = matches!(app_state.input_field, InputField::Question);
    let prefix = if is_active {
        format!("{} ", NeonTheme::SPARKLE)
    } else {
        "   ".to_string()
    };
    let input = Paragraph::new(Line::from(vec![
        Span::styled(prefix, Style::default().fg(NeonTheme::PURPLE_NEON)),
        Span::styled(
            "Вопрос: ",
            Style::default()
                .fg(if is_active {
                    NeonTheme::PURPLE_BRIGHT
                } else {
                    NeonTheme::TEXT_DARK
                })
                .add_modifier(if is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
        Span::styled(
            question,
            Style::default()
                .fg(if is_active {
                    NeonTheme::PURPLE_BRIGHT
                } else {
                    NeonTheme::TEXT_DIM
                })
                .add_modifier(if is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
    ]));
    frame.render_widget(input, *area);
}

fn render_answer_field(
    frame: &mut Frame,
    area: &Rect,
    answer: &str,
    app_state: &AppState,
) {
    let is_active = matches!(app_state.input_field, InputField::Answer);
    let prefix = if is_active {
        format!("{} ", NeonTheme::STAR)
    } else {
        "   ".to_string()
    };
    let input = Paragraph::new(Line::from(vec![
        Span::styled(prefix, Style::default().fg(NeonTheme::GREEN_NEON)),
        Span::styled(
            "Ответ: ",
            Style::default()
                .fg(if is_active {
                    NeonTheme::GREEN_BRIGHT
                } else {
                    NeonTheme::TEXT_DARK
                })
                .add_modifier(if is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
        Span::styled(
            answer,
            Style::default()
                .fg(if is_active {
                    NeonTheme::GREEN_BRIGHT
                } else {
                    NeonTheme::TEXT_DIM
                })
                .add_modifier(if is_active {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
    ]));
    frame.render_widget(input, *area);
}
