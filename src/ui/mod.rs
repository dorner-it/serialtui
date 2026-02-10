mod baud_select;
mod data_bits_select;
mod dialog;
mod menu_bar;
mod parity_select;
mod port_select;
mod status_bar;
mod stop_bits_select;
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
        Screen::DataBitsSelect => data_bits_select::render(app, frame, content_area),
        Screen::ParitySelect => parity_select::render(app, frame, content_area),
        Screen::StopBitsSelect => stop_bits_select::render(app, frame, content_area),
        Screen::Connected => terminal_view::render(app, frame, content_area),
    }

    // Menu bar renders after content so dropdowns overlay
    menu_bar::render(app, frame, menu_area);

    // Dialog renders last, on top of everything
    if let Some(ref dialog) = app.dialog {
        dialog::render(dialog, frame);
    }
}
