use crate::tui::app::AppState;
use crate::tui::ui::common::{create_vertical_layout, render_footer};
use crate::tui::ui::theme::NeonTheme;
use crate::tui::widgets;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

pub fn render(frame: &mut Frame, app_state: &AppState) {
    if let Some(card) = app_state.current_card_in_list() {
        let layout = create_vertical_layout(
            frame.area(),
            &[
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        );

        render_header(frame, &layout[0]);
        render_content(frame, &layout[1], card);
        render_footer(
            frame,
            &layout[2],
            "E: Редактировать | D: Удалить | ESC: Назад",
            NeonTheme::CYAN,
        );
    }
}

fn render_header(frame: &mut Frame, area: &Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", NeonTheme::SPARKLE),
            Style::default()
                .fg(NeonTheme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Просмотр",
            Style::default()
                .fg(NeonTheme::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(header, *area);
}

fn render_content(frame: &mut Frame, area: &Rect, card: &crate::domain::Card) {
    let content = widgets::create_card_widget(card, true);
    frame.render_widget(content, *area);
}
