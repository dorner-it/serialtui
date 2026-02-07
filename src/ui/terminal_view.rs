use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, ViewMode};
use crate::serial::Connection;

pub fn render(app: &App, frame: &mut Frame) {
    if app.connections.is_empty() {
        return;
    }

    let [main_area, input_area, status_area] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    match app.view_mode {
        ViewMode::Tabs => render_tabs(app, frame, main_area),
        ViewMode::Grid => render_grid(app, frame, main_area),
    }

    // Input bar
    let input = Paragraph::new(Line::raw(format!("> {}", app.input_buffer)))
        .block(Block::default().title(" Send ").borders(Borders::ALL));
    frame.render_widget(input, input_area);

    super::status_bar::render(&app.screen, frame, status_area);
}

fn render_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let [tab_bar, content_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(area);

    // Tab bar
    let tabs: Vec<Span> = app
        .connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            let label = format!(" {} ", conn.label());
            if i == app.active_connection {
                Span::styled(
                    label,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                let color = if conn.alive { Color::White } else { Color::Red };
                Span::styled(label, Style::default().fg(color))
            }
        })
        .collect();

    let mut all_spans = tabs;
    all_spans.push(Span::styled(" [+] ", Style::default().fg(Color::Green)));

    frame.render_widget(Paragraph::new(Line::from(all_spans)), tab_bar);

    // Scrollback content
    render_scrollback(
        &app.connections[app.active_connection],
        frame,
        content_area,
        true,
    );
}

fn render_grid(app: &App, frame: &mut Frame, area: Rect) {
    let count = app.connections.len();
    if count == 0 {
        return;
    }

    let cols = (count as f64).sqrt().ceil() as usize;
    let rows = count.div_ceil(cols);

    let row_constraints: Vec<Constraint> = (0..rows)
        .map(|_| Constraint::Ratio(1, rows as u32))
        .collect();
    let row_areas = Layout::vertical(row_constraints).split(area);

    let col_constraints: Vec<Constraint> = (0..cols)
        .map(|_| Constraint::Ratio(1, cols as u32))
        .collect();

    for row in 0..rows {
        let col_areas = Layout::horizontal(col_constraints.clone()).split(row_areas[row]);
        for col in 0..cols {
            let idx = row * cols + col;
            if idx >= count {
                break;
            }
            let is_active = idx == app.active_connection;
            render_scrollback(&app.connections[idx], frame, col_areas[col], is_active);
        }
    }
}

fn render_scrollback(conn: &Connection, frame: &mut Frame, area: Rect, is_active: bool) {
    let border_color = if !conn.alive {
        Color::Red
    } else if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let status = if conn.alive { "" } else { " [DISCONNECTED]" };
    let title = format!(" {}{} ", conn.label(), status);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    if visible_height == 0 {
        return;
    }

    let lines: Vec<&str> = conn.scrollback_with_partial().collect();
    let total = lines.len();

    let start = if total > visible_height + conn.scroll_offset {
        total - visible_height - conn.scroll_offset
    } else {
        0
    };
    let end = total.saturating_sub(conn.scroll_offset);

    let visible_lines: Vec<Line> = lines[start..end].iter().map(|s| Line::raw(*s)).collect();

    let content = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
    frame.render_widget(content, inner);
}
