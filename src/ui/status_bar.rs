use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::Screen;

pub fn render(screen: &Screen, frame: &mut Frame, area: Rect) {
    let help = match screen {
        Screen::PortSelect => "↑↓ Navigate  Enter Select  r Refresh  q Quit",
        Screen::BaudSelect => "↑↓ Navigate  Enter Connect  Esc Back",
        Screen::Connected => {
            "Tab Switch  Ctrl+N New  Ctrl+W Close  Ctrl+G Grid  PgUp/Dn Scroll  Ctrl+Q Quit"
        }
    };

    let bar =
        Paragraph::new(Line::raw(help)).style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(bar, area);
}
