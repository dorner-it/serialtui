use std::sync::mpsc;

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

    fn connection_by_id(&mut self, id: usize) -> Option<&mut Connection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }
}
