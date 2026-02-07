use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{App, OpenMenu};

const NORMAL: Style = Style::new().fg(Color::Black).bg(Color::White);
const HIGHLIGHT: Style = Style::new()
    .fg(Color::White)
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD);

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let file_style = if app.open_menu == Some(OpenMenu::File) {
        HIGHLIGHT
    } else {
        NORMAL
    };
    let conn_style = if app.open_menu == Some(OpenMenu::Connection) {
        HIGHLIGHT
    } else {
        NORMAL
    };
    let view_style = if app.open_menu == Some(OpenMenu::View) {
        HIGHLIGHT
    } else {
        NORMAL
    };

    let bar = Line::from(vec![
        Span::styled(" File ", file_style),
        Span::styled(" Connection ", conn_style),
        Span::styled(" View ", view_style),
    ]);

    let bg = Paragraph::new(bar).style(NORMAL);
    frame.render_widget(bg, area);

    // Render dropdown if a menu is open
    if let Some(menu) = app.open_menu {
        let frame_area = frame.area();
        match menu {
            OpenMenu::File => {
                render_dropdown(
                    frame,
                    1,
                    1,
                    &[" Export       ", " Quit         "],
                    frame_area,
                );
            }
            OpenMenu::Connection => {
                render_dropdown(
                    frame,
                    7,
                    1,
                    &[" New          ", " Close        "],
                    frame_area,
                );
            }
            OpenMenu::View => {
                render_dropdown(
                    frame,
                    19,
                    1,
                    &[" Tab View     ", " Grid View    "],
                    frame_area,
                );
            }
        }
    }
}

fn render_dropdown(frame: &mut Frame, x: u16, y: u16, items: &[&str], frame_area: Rect) {
    let width = 16_u16;
    let height = items.len() as u16 + 2; // +2 for border

    if x + width > frame_area.width || y + height > frame_area.height {
        return;
    }

    let area = Rect::new(x, y, width, height);

    // Clear the area behind the dropdown
    frame.render_widget(Clear, area);

    let lines: Vec<Line> = items.iter().map(|s| Line::raw(*s)).collect();

    let dropdown = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::Black).bg(Color::White));

    frame.render_widget(dropdown, area);
}
