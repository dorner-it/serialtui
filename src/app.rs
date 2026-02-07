use std::sync::mpsc;
use std::time::Instant;

use crate::message::Message;
use crate::serial::{Connection, SerialEvent};

pub const BAUD_RATES: &[u32] = &[
    300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
];

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    PortSelect,
    BaudSelect,
    Connected,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    Tabs,
    Grid,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OpenMenu {
    File,
    Connection,
    View,
}

// Menu bar layout constants — must match menu_bar.rs rendering
pub const MENU_FILE_X: u16 = 1;
pub const MENU_FILE_W: u16 = 6; // " File "
pub const MENU_CONN_X: u16 = 7;
pub const MENU_CONN_W: u16 = 12; // " Connection "
pub const MENU_VIEW_X: u16 = 19;
pub const MENU_VIEW_W: u16 = 6; // " View "

pub struct PortInfo {
    pub name: String,
    pub description: String,
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,

    // Port selection
    pub available_ports: Vec<PortInfo>,
    pub selected_port_index: usize,

    // Baud selection
    pub selected_baud_index: usize,

    // Connections
    pub connections: Vec<Connection>,
    pub active_connection: usize,
    pub view_mode: ViewMode,

    // Input
    pub input_buffer: String,

    // Serial channel
    pub serial_tx: mpsc::Sender<SerialEvent>,
    pub serial_rx: mpsc::Receiver<SerialEvent>,

    // ID counter
    next_connection_id: usize,

    // Returning from new-connection flow
    pub adding_connection: bool,

    // Status message (shown briefly in status bar)
    pub status_message: Option<(String, Instant)>,

    // Menu
    pub open_menu: Option<OpenMenu>,
}

impl App {
    pub fn new() -> Self {
        let (serial_tx, serial_rx) = mpsc::channel();

        let mut app = Self {
            screen: Screen::PortSelect,
            should_quit: false,
            available_ports: Vec::new(),
            selected_port_index: 0,
            selected_baud_index: 4, // 9600 default
            connections: Vec::new(),
            active_connection: 0,
            view_mode: ViewMode::Tabs,
            input_buffer: String::new(),
            serial_tx,
            serial_rx,
            next_connection_id: 0,
            adding_connection: false,
            status_message: None,
            open_menu: None,
        };
        app.refresh_ports();
        app
    }

    pub fn refresh_ports(&mut self) {
        self.available_ports = match serialport::available_ports() {
            Ok(ports) => ports
                .into_iter()
                .map(|p| {
                    let description = match &p.port_type {
                        serialport::SerialPortType::UsbPort(info) => {
                            info.product.clone().unwrap_or_else(|| "USB Serial".into())
                        }
                        serialport::SerialPortType::BluetoothPort => "Bluetooth".into(),
                        serialport::SerialPortType::PciPort => "PCI".into(),
                        serialport::SerialPortType::Unknown => String::new(),
                    };
                    PortInfo {
                        name: p.port_name,
                        description,
                    }
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        if self.selected_port_index >= self.available_ports.len() {
            self.selected_port_index = 0;
        }
    }

    pub fn drain_serial_events(&mut self) {
        while let Ok(event) = self.serial_rx.try_recv() {
            match event {
                SerialEvent::Data { id, data } => {
                    if let Some(conn) = self.connection_by_id(id) {
                        conn.push_data(&data);
                    }
                }
                SerialEvent::Error { id, err } => {
                    if let Some(conn) = self.connection_by_id(id) {
                        conn.push_data(format!("\n[ERROR: {}]\n", err).as_bytes());
                        conn.alive = false;
                    }
                }
                SerialEvent::Disconnected { id } => {
                    if let Some(conn) = self.connection_by_id(id) {
                        conn.push_data(b"\n[DISCONNECTED]\n");
                        conn.alive = false;
                    }
                }
            }
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Quit => self.should_quit = true,

            Message::Up => match self.screen {
                Screen::PortSelect => {
                    if self.selected_port_index > 0 {
                        self.selected_port_index -= 1;
                    }
                }
                Screen::BaudSelect => {
                    if self.selected_baud_index > 0 {
                        self.selected_baud_index -= 1;
                    }
                }
                _ => {}
            },

            Message::Down => match self.screen {
                Screen::PortSelect => {
                    if !self.available_ports.is_empty()
                        && self.selected_port_index < self.available_ports.len() - 1
                    {
                        self.selected_port_index += 1;
                    }
                }
                Screen::BaudSelect => {
                    if self.selected_baud_index < BAUD_RATES.len() - 1 {
                        self.selected_baud_index += 1;
                    }
                }
                _ => {}
            },

            Message::Select => match self.screen {
                Screen::PortSelect => {
                    if !self.available_ports.is_empty() {
                        self.screen = Screen::BaudSelect;
                    }
                }
                Screen::BaudSelect => {
                    self.connect_selected();
                }
                _ => {}
            },

            Message::Back => match self.screen {
                Screen::PortSelect => {
                    if self.adding_connection {
                        self.adding_connection = false;
                        self.screen = Screen::Connected;
                    }
                }
                Screen::BaudSelect => {
                    if self.adding_connection {
                        self.adding_connection = false;
                        self.screen = Screen::Connected;
                    } else {
                        self.screen = Screen::PortSelect;
                    }
                }
                _ => {}
            },

            Message::RefreshPorts => {
                self.refresh_ports();
            }

            Message::NewConnection => {
                if self.screen == Screen::Connected {
                    self.adding_connection = true;
                    self.refresh_ports();
                    self.screen = Screen::PortSelect;
                }
            }

            Message::CloseConnection => {
                if !self.connections.is_empty() {
                    let idx = self.active_connection;
                    self.connections[idx].close();
                    self.connections.remove(idx);
                    if self.connections.is_empty() {
                        self.screen = Screen::PortSelect;
                        self.adding_connection = false;
                        self.refresh_ports();
                    } else if self.active_connection >= self.connections.len() {
                        self.active_connection = self.connections.len() - 1;
                    }
                }
            }

            Message::NextTab => {
                if !self.connections.is_empty() {
                    self.active_connection = (self.active_connection + 1) % self.connections.len();
                }
            }

            Message::PrevTab => {
                if !self.connections.is_empty() {
                    self.active_connection = if self.active_connection == 0 {
                        self.connections.len() - 1
                    } else {
                        self.active_connection - 1
                    };
                }
            }

            Message::SwitchTab(n) => {
                if n < self.connections.len() {
                    self.active_connection = n;
                }
            }

            Message::ToggleViewMode => {
                self.view_mode = match self.view_mode {
                    ViewMode::Tabs => ViewMode::Grid,
                    ViewMode::Grid => ViewMode::Tabs,
                };
            }

            Message::CharInput(c) => {
                self.input_buffer.push(c);
            }

            Message::Backspace => {
                self.input_buffer.pop();
            }

            Message::SendInput => {
                if !self.input_buffer.is_empty() && !self.connections.is_empty() {
                    let data = format!("{}\r\n", self.input_buffer);
                    self.connections[self.active_connection].send(data.as_bytes());
                    self.input_buffer.clear();
                }
            }

            Message::ExportScrollback => {
                if !self.connections.is_empty() {
                    self.export_active_scrollback();
                }
            }

            Message::ScrollUp => {
                if !self.connections.is_empty() {
                    let conn = &mut self.connections[self.active_connection];
                    let total = conn.scrollback.len();
                    if conn.scroll_offset < total {
                        conn.scroll_offset = (conn.scroll_offset + 5).min(total);
                    }
                }
            }

            Message::ScrollDown => {
                if !self.connections.is_empty() {
                    let conn = &mut self.connections[self.active_connection];
                    conn.scroll_offset = conn.scroll_offset.saturating_sub(5);
                }
            }

            Message::CloseMenu => {
                self.open_menu = None;
            }

            Message::MenuClick(col, row) => {
                self.handle_menu_click(col, row);
            }
        }
    }

    fn handle_menu_click(&mut self, col: u16, row: u16) {
        let file_range = MENU_FILE_X..MENU_FILE_X + MENU_FILE_W;
        let conn_range = MENU_CONN_X..MENU_CONN_X + MENU_CONN_W;
        let view_range = MENU_VIEW_X..MENU_VIEW_X + MENU_VIEW_W;

        if row == 0 {
            // Clicking on the menu bar itself — toggle menus
            let new_menu = if file_range.contains(&col) {
                Some(OpenMenu::File)
            } else if conn_range.contains(&col) {
                Some(OpenMenu::Connection)
            } else if view_range.contains(&col) {
                Some(OpenMenu::View)
            } else {
                None
            };
            if new_menu == self.open_menu {
                self.open_menu = None;
            } else {
                self.open_menu = new_menu;
            }
            return;
        }

        // Clicking on an open dropdown
        let Some(menu) = self.open_menu else {
            return;
        };

        let drop_w = 0..16_u16; // dropdown is 16 chars wide
        let handled = match menu {
            OpenMenu::File => {
                let drop_col = col.wrapping_sub(MENU_FILE_X);
                if row == 1 && drop_w.contains(&drop_col) {
                    self.open_menu = None;
                    if !self.connections.is_empty() && self.screen == Screen::Connected {
                        self.export_active_scrollback();
                    }
                    true
                } else if row == 2 && drop_w.contains(&drop_col) {
                    self.should_quit = true;
                    true
                } else {
                    false
                }
            }
            OpenMenu::Connection => {
                let drop_col = col.wrapping_sub(MENU_CONN_X);
                if row == 1 && drop_w.contains(&drop_col) {
                    self.open_menu = None;
                    if self.screen == Screen::Connected {
                        self.adding_connection = true;
                        self.refresh_ports();
                        self.screen = Screen::PortSelect;
                    }
                    true
                } else if row == 2 && drop_w.contains(&drop_col) {
                    self.open_menu = None;
                    if !self.connections.is_empty() {
                        let idx = self.active_connection;
                        self.connections[idx].close();
                        self.connections.remove(idx);
                        if self.connections.is_empty() {
                            self.screen = Screen::PortSelect;
                            self.adding_connection = false;
                            self.refresh_ports();
                        } else if self.active_connection >= self.connections.len() {
                            self.active_connection = self.connections.len() - 1;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            OpenMenu::View => {
                let drop_col = col.wrapping_sub(MENU_VIEW_X);
                if row == 1 && drop_w.contains(&drop_col) {
                    self.open_menu = None;
                    self.view_mode = ViewMode::Tabs;
                    true
                } else if row == 2 && drop_w.contains(&drop_col) {
                    self.open_menu = None;
                    self.view_mode = ViewMode::Grid;
                    true
                } else {
                    false
                }
            }
        };
        if !handled {
            self.open_menu = None;
        }
    }

    fn connect_selected(&mut self) {
        if self.available_ports.is_empty() {
            return;
        }
        let port_name = self.available_ports[self.selected_port_index].name.clone();
        let baud_rate = BAUD_RATES[self.selected_baud_index];
        let id = self.next_connection_id;
        self.next_connection_id += 1;

        let conn = Connection::new(id, port_name, baud_rate, self.serial_tx.clone());
        self.connections.push(conn);
        self.active_connection = self.connections.len() - 1;
        self.adding_connection = false;
        self.screen = Screen::Connected;
    }

    fn export_active_scrollback(&mut self) {
        let conn = &self.connections[self.active_connection];
        let safe_name = conn.port_name.replace(['/', '\\', ':'], "_");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let filename = format!("{}_{}_{}.txt", safe_name, conn.baud_rate, timestamp);

        let content: String = conn
            .scrollback_with_partial()
            .collect::<Vec<_>>()
            .join("\n");

        match std::fs::write(&filename, &content) {
            Ok(()) => {
                self.status_message = Some((format!("Exported to {}", filename), Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((format!("Export failed: {}", e), Instant::now()));
            }
        }
    }

    pub fn status_text(&self) -> Option<&str> {
        if let Some((msg, time)) = &self.status_message {
            if time.elapsed().as_secs() < 3 {
                return Some(msg);
            }
        }
        None
    }

    fn connection_by_id(&mut self, id: usize) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }
}
