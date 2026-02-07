use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::Dialog;

pub fn render(dialog: &Dialog, frame: &mut Frame) {
    match dialog {
        Dialog::ConfirmCloseConnection => {
            render_confirm(
                frame,
                " Close Connection ",
                "Save session before closing?",
                "[Y]es  [N]o  [Esc] Cancel",
            );
        }
        Dialog::ConfirmQuit => {
            render_confirm(
                frame,
                " Quit ",
                "Export all open sessions before quitting?",
                "[Y]es  [N]o  [Esc] Cancel",
            );
        }
        Dialog::FileNamePrompt { filename, .. } => {
            render_filename_prompt(frame, filename);
        }
    }
}

fn center_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [_, varea, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .flex(Flex::Center)
    .areas(area);

    let [_, harea, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .flex(Flex::Center)
    .areas(varea);

    harea
}

fn render_confirm(frame: &mut Frame, title: &str, message: &str, hint: &str) {
    let width = (message.len() as u16 + 4)
        .max(hint.len() as u16 + 4)
        .max(30);
    let area = center_rect(width, 5, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [msg_area, hint_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);

    let msg = Paragraph::new(Line::raw(message)).style(Style::default().fg(Color::White));
    frame.render_widget(msg, msg_area);

    let hints = Paragraph::new(Line::raw(hint)).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(hints, hint_area);
}

fn render_filename_prompt(frame: &mut Frame, filename: &str) {
    let width = (filename.len() as u16 + 6).max(40);
    let area = center_rect(width, 6, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Export Filename ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [label_area, input_area, hint_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    let label = Paragraph::new(Line::raw("Filename (edit or press Enter):"))
        .style(Style::default().fg(Color::White));
    frame.render_widget(label, label_area);

    let input = Paragraph::new(Line::raw(format!("> {}", filename))).style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(input, input_area);

    let hints = Paragraph::new(Line::raw("Enter Confirm  Esc Cancel"))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hints, hint_area);
}
