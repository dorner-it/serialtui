use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, DATA_BITS_OPTIONS};

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let [main_area, status_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);

    let port_name = app
        .available_ports
        .get(app.selected_port_index)
        .map(|p| p.name.as_str())
        .unwrap_or("?");

    let items: Vec<ListItem> = DATA_BITS_OPTIONS
        .iter()
        .map(|(label, _)| ListItem::new(Line::raw(*label)))
        .collect();

    let title = format!(" Data Bits for {} ", port_name);
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default().with_selected(Some(app.selected_data_bits_index));
    frame.render_stateful_widget(list, main_area, &mut state);

    super::status_bar::render(app, frame, status_area);
}

/// Render just the data bits list (no status bar, no outer block) for inline use in tabs/grid.
pub fn render_content(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = DATA_BITS_OPTIONS
        .iter()
        .map(|(label, _)| ListItem::new(Line::raw(*label)))
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default().with_selected(Some(app.selected_data_bits_index));
    frame.render_stateful_widget(list, area, &mut state);
}
