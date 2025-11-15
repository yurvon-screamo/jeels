mod card_create;
mod card_edit;
mod card_list;
mod card_view;
mod common;
mod main_menu;
mod study_session;
pub mod theme;

pub use theme::NeonTheme;

use crate::tui::app::AppState;
use crate::tui::app::Screen;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app_state: &AppState) {
    match app_state.screen {
        Screen::MainMenu => main_menu::render(frame, app_state),
        Screen::StudySession => study_session::render(frame, app_state),
        Screen::CardList => card_list::render(frame, app_state),
        Screen::CardView { .. } => card_view::render(frame, app_state),
        Screen::CardEdit { .. } => card_edit::render(frame, app_state),
        Screen::CardCreate { .. } => card_create::render(frame, app_state),
    }
}
