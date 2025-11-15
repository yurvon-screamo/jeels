use crate::tui::app::AppState;
use crate::tui::ui::common::create_vertical_layout;
use crate::tui::ui::theme::NeonTheme;
use crate::tui::widgets;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

pub fn render(frame: &mut Frame, app_state: &AppState) {
    let layout = create_vertical_layout(
        frame.area(),
        &[
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
        ],
    );

    render_header(frame, &layout[0], app_state);
    render_card_content(frame, &layout[1], app_state);
    render_separator(frame, &layout[2]);
    render_instructions(frame, &layout[3], app_state);
}

fn render_header(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", NeonTheme::SPARKLE),
            Style::default()
                .fg(NeonTheme::PURPLE_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "[",
            Style::default().fg(NeonTheme::PURPLE_BRIGHT),
        ),
        Span::styled(
            format!("{}/{}", app_state.current_index + 1, app_state.cards_count()),
            Style::default()
                .fg(NeonTheme::GREEN_NEON)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "]",
            Style::default().fg(NeonTheme::PURPLE_BRIGHT),
        ),
    ]));
    frame.render_widget(header, *area);
}

fn render_card_content(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let card_widget = if let Some(card) = app_state.current_card() {
        widgets::create_card_widget(card, app_state.show_answer)
    } else {
        widgets::create_empty_card_widget()
    };
    frame.render_widget(card_widget, *area);
}

fn render_separator(frame: &mut Frame, area: &Rect) {
    let separator = Paragraph::new("─".repeat(area.width as usize))
        .style(Style::default().fg(NeonTheme::TEXT_DARK));
    frame.render_widget(separator, *area);
}

fn render_instructions(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let instructions_lines = widgets::get_instructions_text(app_state.show_answer);
    let instructions = Paragraph::new(instructions_lines);
    frame.render_widget(instructions, *area);
}
