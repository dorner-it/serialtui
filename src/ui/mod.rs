mod baud_select;
mod port_select;
mod status_bar;
mod terminal_view;

use ratatui::Frame;

use crate::app::{App, Screen};

pub fn render(app: &App, frame: &mut Frame) {
    match app.screen {
        Screen::PortSelect => port_select::render(app, frame),
        Screen::BaudSelect => baud_select::render(app, frame),
        Screen::Connected => terminal_view::render(app, frame),
    }
}
