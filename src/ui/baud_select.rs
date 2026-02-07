use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, BAUD_RATES};

pub fn render(app: &App, frame: &mut Frame) {
    let [main_area, status_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

    let port_name = app
        .available_ports
        .get(app.selected_port_index)
        .map(|p| p.name.as_str())
        .unwrap_or("?");

    let items: Vec<ListItem> = BAUD_RATES
        .iter()
        .map(|b| ListItem::new(Line::raw(b.to_string())))
        .collect();

    let title = format!(" Baud Rate for {} ", port_name);
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default().with_selected(Some(app.selected_baud_index));
    frame.render_stateful_widget(list, main_area, &mut state);

    super::status_bar::render(app, frame, status_area);
}
