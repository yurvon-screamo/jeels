use crate::tui::ui::theme::NeonTheme;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};
use std::rc::Rc;

pub fn create_vertical_layout(area: Rect, constraints: &[Constraint]) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
}

pub fn render_header(frame: &mut Frame, area: &Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", NeonTheme::SPARKLE),
            Style::default()
                .fg(NeonTheme::PURPLE_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "J",
            Style::default()
                .fg(NeonTheme::PURPLE_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "E",
            Style::default()
                .fg(NeonTheme::GREEN_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "E",
            Style::default()
                .fg(NeonTheme::PURPLE_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "L",
            Style::default()
                .fg(NeonTheme::GREEN_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "S",
            Style::default()
                .fg(NeonTheme::PURPLE_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " CLI",
            Style::default()
                .fg(NeonTheme::GREEN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", NeonTheme::STAR),
            Style::default()
                .fg(NeonTheme::MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(title, *area);
}

pub fn render_footer(frame: &mut Frame, area: &Rect, text: &str, _color: Color) {
    let footer = Paragraph::new(text)
        .style(Style::default().fg(NeonTheme::TEXT_DIM));
    frame.render_widget(footer, *area);
}
