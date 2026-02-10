use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(status) = app.status_text() {
        let bar = Paragraph::new(Line::raw(status))
            .style(Style::default().fg(Color::Black).bg(Color::Green));
        frame.render_widget(bar, area);
        return;
    }

    let help = match app.screen {
        crate::app::Screen::PortSelect => "↑↓ Navigate  Enter Select  r Refresh  Esc/q Quit",
        crate::app::Screen::BaudSelect => "↑↓ Navigate  Enter Select  Esc Back",
        crate::app::Screen::DataBitsSelect => "↑↓ Navigate  Enter Select  Esc Back",
        crate::app::Screen::ParitySelect => "↑↓ Navigate  Enter Select  Esc Back",
        crate::app::Screen::StopBitsSelect => "↑↓ Navigate  Enter Connect  Esc Back",
        crate::app::Screen::Connected => {
            if app.is_pending_active() {
                match app.pending_connection {
                    Some(crate::app::PendingScreen::PortSelect) => {
                        "↑↓ Navigate  Enter Select  r Refresh  Tab Switch  Esc Cancel"
                    }
                    Some(crate::app::PendingScreen::BaudSelect) => {
                        "↑↓ Navigate  Enter Select  Tab Switch  Esc Back"
                    }
                    Some(crate::app::PendingScreen::DataBitsSelect) => {
                        "↑↓ Navigate  Enter Select  Tab Switch  Esc Back"
                    }
                    Some(crate::app::PendingScreen::ParitySelect) => {
                        "↑↓ Navigate  Enter Select  Tab Switch  Esc Back"
                    }
                    Some(crate::app::PendingScreen::StopBitsSelect) => {
                        "↑↓ Navigate  Enter Connect  Tab Switch  Esc Back"
                    }
                    None => "",
                }
            } else {
                "Tab Switch  Ctrl+N New  Ctrl+W Close  Ctrl+E Export  Ctrl+G Grid  ↑↓/PgUp/Dn/Wheel Scroll  Ctrl+Q Quit"
            }
        }
    };

    let bar =
        Paragraph::new(Line::raw(help)).style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(bar, area);
}
