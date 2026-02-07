mod baud_select;
mod menu_bar;
mod port_select;
mod status_bar;
mod terminal_view;

use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn render(app: &App, frame: &mut Frame) {
    let [menu_area, content_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(frame.area());

    match app.screen {
        Screen::PortSelect => port_select::render(app, frame, content_area),
        Screen::BaudSelect => baud_select::render(app, frame, content_area),
        Screen::Connected => terminal_view::render(app, frame, content_area),
    }

    // Menu bar renders last so dropdowns overlay content
    menu_bar::render(app, frame, menu_area);
}
