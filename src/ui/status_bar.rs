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
        crate::app::Screen::PortSelect => "↑↓ Navigate  Enter Select  r Refresh  q Quit",
        crate::app::Screen::BaudSelect => "↑↓ Navigate  Enter Connect  Esc Back",
        crate::app::Screen::Connected => {
            "Tab Switch  Ctrl+N New  Ctrl+W Close  Ctrl+E Export  Ctrl+G Grid  PgUp/Dn Scroll  Ctrl+Q Quit"
        }
    };

    let bar =
        Paragraph::new(Line::raw(help)).style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(bar, area);
}
