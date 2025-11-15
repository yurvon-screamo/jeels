use crate::tui::app::{AppState, MenuOption};
use crate::tui::ui::common::{create_vertical_layout, render_footer, render_header};
use crate::tui::ui::theme::NeonTheme;
use ratatui::{
    prelude::*,
    widgets::{List, ListItem},
};

pub fn render(frame: &mut Frame, app_state: &AppState) {
    let area = frame.area();
    let layout = create_vertical_layout(
        area,
        &[
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Length(1),
        ],
    );

    render_header(frame, &layout[0]);
    render_menu_options(frame, &layout[1], app_state);
    render_footer(
        frame,
        &layout[2],
        "↑↓: Навигация | Enter: Выбрать | Q/ESC: Выход",
        NeonTheme::PURPLE_NEON,
    );
}

fn render_menu_options(frame: &mut Frame, area: &Rect, app_state: &AppState) {
    let options = vec![
        (
            MenuOption::Study,
            format!("{} Учеба", NeonTheme::SPARKLE),
            "Начать сессию изучения карточек",
            NeonTheme::PURPLE_BRIGHT,
            NeonTheme::GREEN_NEON,
        ),
        (
            MenuOption::ManageCards,
            format!("{} Управление карточками", NeonTheme::DIAMOND),
            "Просмотр, редактирование и удаление карточек",
            NeonTheme::GREEN_NEON,
            NeonTheme::PURPLE_BRIGHT,
        ),
        (
            MenuOption::CreateCard,
            format!("{} Создать новую карточку", NeonTheme::STAR),
            "Добавить новую карточку для изучения",
            NeonTheme::MAGENTA,
            NeonTheme::CYAN,
        ),
    ];

    let items: Vec<ListItem> = options
        .iter()
        .map(|(option, title, desc, primary_color, accent_color)| {
            let is_selected = app_state.selected_menu_option == *option;
            let title_style = if is_selected {
                Style::default()
                    .fg(*primary_color)
                    .bg(NeonTheme::BG_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default()
                    .fg(*primary_color)
                    .add_modifier(Modifier::BOLD)
            };

            let arrow = if is_selected {
                format!("{} ", NeonTheme::SPARKLE)
            } else {
                "  ".to_string()
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(arrow, Style::default().fg(*accent_color)),
                    Span::styled(title.as_str(), title_style),
                ]),
                Line::from(vec![Span::styled(
                    format!("   {}", desc),
                    Style::default().fg(if is_selected {
                        NeonTheme::TEXT_DIM
                    } else {
                        NeonTheme::TEXT_DARK
                    }),
                )]),
            ])
        })
        .collect();

    let list = List::new(items)
        .style(Style::default().fg(NeonTheme::TEXT_BRIGHT));

    frame.render_widget(list, *area);
}
