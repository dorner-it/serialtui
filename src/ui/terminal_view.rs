use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use crate::app::{App, PendingScreen, ViewMode};
use crate::serial::Connection;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    if app.connections.is_empty() && app.pending_connection.is_none() {
        return;
    }

    let [main_area, input_area, status_area] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(area);

    match app.view_mode {
        ViewMode::Tabs => render_tabs(app, frame, main_area),
        ViewMode::Grid => render_grid(app, frame, main_area),
    }

    // Input bar
    let input = Paragraph::new(Line::raw(format!("> {}", app.input_buffer)))
        .block(Block::default().title(" Send ").borders(Borders::ALL));
    frame.render_widget(input, input_area);

    super::status_bar::render(app, frame, status_area);
}

fn render_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let [tab_bar, content_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(area);

    // Tab bar
    let mut all_spans: Vec<Span> = app
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

    // "New" tab when a pending connection exists
    if app.pending_connection.is_some() {
        let pending_idx = app.connections.len();
        let style = if app.active_connection == pending_idx {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        all_spans.push(Span::styled(" New ", style));
    } else {
        all_spans.push(Span::styled(" [+] ", Style::default().fg(Color::Green)));
    }

    frame.render_widget(Paragraph::new(Line::from(all_spans)), tab_bar);

    // Content area
    if app.is_pending_active() {
        render_pending_cell(app, frame, content_area, true);
    } else if app.active_connection < app.connections.len() {
        render_scrollback(
            &app.connections[app.active_connection],
            frame,
            content_area,
            true,
        );
    }
}

fn render_grid(app: &App, frame: &mut Frame, area: Rect) {
    let total = app.connections.len()
        + if app.pending_connection.is_some() {
            1
        } else {
            0
        };
    if total == 0 {
        return;
    }

    let cols = (total as f64).sqrt().ceil() as usize;
    let rows = total.div_ceil(cols);

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
            if idx >= total {
                break;
            }
            if idx < app.connections.len() {
                let is_active = idx == app.active_connection;
                render_scrollback(&app.connections[idx], frame, col_areas[col], is_active);
            } else {
                let is_active = app.active_connection == app.connections.len();
                render_pending_cell(app, frame, col_areas[col], is_active);
            }
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

    // Clamp offset so the top of scrollback always fills the visible area
    let max_offset = total.saturating_sub(visible_height);
    let offset = conn.scroll_offset.min(max_offset);

    let start = if total > visible_height + offset {
        total - visible_height - offset
    } else {
        0
    };
    let end = total.saturating_sub(offset);

    let visible_lines: Vec<Line> = lines[start..end].iter().map(|s| Line::raw(*s)).collect();

    let content = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
    frame.render_widget(content, inner);

    // Scrollbar
    if total > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(total)
            .position(start)
            .viewport_content_length(visible_height);
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn render_pending_cell(app: &App, frame: &mut Frame, area: Rect, is_active: bool) {
    let pending = match app.pending_connection {
        Some(p) => p,
        None => return,
    };

    let border_color = if is_active {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let title = match pending {
        PendingScreen::PortSelect => " Select Port ",
        PendingScreen::BaudSelect => " Select Baud ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match pending {
        PendingScreen::PortSelect => {
            super::port_select::render_content(app, frame, inner);
        }
        PendingScreen::BaudSelect => {
            super::baud_select::render_content(app, frame, inner);
        }
    }
}
