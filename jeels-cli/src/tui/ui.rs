use crate::tui::app::AppState;
use crate::tui::widgets;
use ratatui::{
    Frame,
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::rc::Rc;

pub fn render(frame: &mut Frame, app_state: &AppState) {
    let layout = create_layout(frame.area());
    render_header(frame, &layout[0], app_state);
    render_card_content(frame, &layout[1], app_state);
    render_instructions(frame, &layout[2], app_state);
    render_status(frame, &layout[3], app_state);
}

fn create_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(5),
        ])
        .split(area)
}

fn render_header(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let header_text = format!(
        "Card {}/{}",
        app_state.current_index + 1,
        app_state.cards_count()
    );
    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Study Session"),
    );
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

fn render_instructions(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let instructions_text = widgets::get_instructions_text(app_state.show_answer);
    let instructions = Paragraph::new(instructions_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    frame.render_widget(instructions, *area);
}

fn render_status(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let status_text = widgets::get_status_text(app_state.cards.is_empty());
    let status =
        Paragraph::new(status_text).block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, *area);
}
