use std::time::Duration;

use ratatui::crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::app::{App, Screen};
use crate::message::Message;

pub fn poll_event(app: &App) -> Option<Message> {
    if !event::poll(Duration::from_millis(50)).ok()? {
        return None;
    }

    let event = event::read().ok()?;

    match event {
        Event::Key(key) => {
            // Ignore key release events (Windows sends both press and release)
            if key.kind != event::KeyEventKind::Press {
                return None;
            }

            // If a menu is open, Esc closes it; otherwise forward to screen handler
            if app.open_menu.is_some() {
                return match key.code {
                    KeyCode::Esc => Some(Message::CloseMenu),
                    _ => Some(Message::CloseMenu),
                };
            }

            match app.screen {
                Screen::PortSelect => map_port_select(key),
                Screen::BaudSelect => map_baud_select(key),
                Screen::Connected => map_connected(key),
            }
        }
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                Some(Message::MenuClick(mouse.column, mouse.row))
            }
            MouseEventKind::ScrollUp => {
                if app.screen == Screen::Connected {
                    Some(Message::ScrollUp)
                } else {
                    None
                }
            }
            MouseEventKind::ScrollDown => {
                if app.screen == Screen::Connected {
                    Some(Message::ScrollDown)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

fn map_port_select(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('r') => Some(Message::RefreshPorts),
        KeyCode::Esc => Some(Message::Back),
        KeyCode::Up => Some(Message::Up),
        KeyCode::Down => Some(Message::Down),
        KeyCode::Enter => Some(Message::Select),
        _ => None,
    }
}

fn map_baud_select(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc => Some(Message::Back),
        KeyCode::Up => Some(Message::Up),
        KeyCode::Down => Some(Message::Down),
        KeyCode::Enter => Some(Message::Select),
        _ => None,
    }
}

fn map_connected(key: KeyEvent) -> Option<Message> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    if ctrl {
        return match key.code {
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('n') => Some(Message::NewConnection),
            KeyCode::Char('w') => Some(Message::CloseConnection),
            KeyCode::Char('g') => Some(Message::ToggleViewMode),
            KeyCode::Char('e') => Some(Message::ExportScrollback),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Tab if shift => Some(Message::PrevTab),
        KeyCode::BackTab => Some(Message::PrevTab),
        KeyCode::Tab => Some(Message::NextTab),
        KeyCode::Char(c @ '1'..='9') => Some(Message::SwitchTab(c as usize - '1' as usize)),
        KeyCode::Up => Some(Message::ScrollUp),
        KeyCode::Down => Some(Message::ScrollDown),
        KeyCode::PageUp => Some(Message::ScrollUp),
        KeyCode::PageDown => Some(Message::ScrollDown),
        KeyCode::Enter => Some(Message::SendInput),
        KeyCode::Backspace => Some(Message::Backspace),
        KeyCode::Char(c) => Some(Message::CharInput(c)),
        _ => None,
    }
}
