use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame) {
    let [main_area, status_area] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

    if app.available_ports.is_empty() {
        let msg = Paragraph::new("No serial ports found. Press 'r' to refresh.").block(
            Block::default()
                .title(" Serial Ports ")
                .borders(Borders::ALL),
        );
        frame.render_widget(msg, main_area);
    } else {
        let items: Vec<ListItem> = app
            .available_ports
            .iter()
            .map(|p| {
                let text = if p.description.is_empty() {
                    p.name.clone()
                } else {
                    format!("{} — {}", p.name, p.description)
                };
                ListItem::new(Line::raw(text))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Select Port ")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut state = ListState::default().with_selected(Some(app.selected_port_index));
        frame.render_stateful_widget(list, main_area, &mut state);
    }

    super::status_bar::render(app, frame, status_area);
}
