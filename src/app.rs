use std::sync::mpsc;
use std::time::Instant;

use crate::message::Message;
use crate::serial::{Connection, SerialEvent};

pub const BAUD_RATES: &[u32] = &[
    300, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
];

pub const PARITY_OPTIONS: &[(&str, serialport::Parity)] = &[
    ("None", serialport::Parity::None),
    ("Odd", serialport::Parity::Odd),
    ("Even", serialport::Parity::Even),
];

pub const DATA_BITS_OPTIONS: &[(&str, serialport::DataBits)] = &[
    ("5", serialport::DataBits::Five),
    ("6", serialport::DataBits::Six),
    ("7", serialport::DataBits::Seven),
    ("8", serialport::DataBits::Eight),
];

pub const STOP_BITS_OPTIONS: &[(&str, serialport::StopBits)] = &[
    ("1", serialport::StopBits::One),
    ("2", serialport::StopBits::Two),
];

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    PortSelect,
    BaudSelect,
    DataBitsSelect,
    ParitySelect,
    StopBitsSelect,
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

#[derive(Clone, Copy, PartialEq)]
pub enum PendingScreen {
    PortSelect,
    BaudSelect,
    DataBitsSelect,
    ParitySelect,
    StopBitsSelect,
}

#[derive(Clone)]
pub enum Dialog {
    ConfirmCloseConnection,
    ConfirmQuit,
    FileNamePrompt {
        connection_idx: usize,
        filename: String,
        cursor_pos: usize,
        after: AfterSave,
    },
}

#[derive(Clone)]
pub enum AfterSave {
    Nothing,
    CloseConnection,
    QuitNext { remaining: Vec<usize> },
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

    // Data bits selection
    pub selected_data_bits_index: usize,

    // Parity selection
    pub selected_parity_index: usize,

    // Stop bits selection
    pub selected_stop_bits_index: usize,

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

    // Inline new-connection flow (shown as a tab/grid cell)
    pub pending_connection: Option<PendingScreen>,

    // Status message (shown briefly in status bar)
    pub status_message: Option<(String, Instant)>,

    // Menu
    pub open_menu: Option<OpenMenu>,

    // Dialog
    pub dialog: Option<Dialog>,

    // Terminal size (updated each frame for click calculations)
    pub terminal_cols: u16,
    pub terminal_rows: u16,
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
            selected_data_bits_index: 3, // Eight
            selected_parity_index: 0,    // None
            selected_stop_bits_index: 0, // One
            connections: Vec::new(),
            active_connection: 0,
            view_mode: ViewMode::Tabs,
            input_buffer: String::new(),
            serial_tx,
            serial_rx,
            next_connection_id: 0,
            pending_connection: None,
            status_message: None,
            open_menu: None,
            dialog: None,
            terminal_cols: 80,
            terminal_rows: 24,
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

    pub fn is_pending_active(&self) -> bool {
        self.pending_connection.is_some() && self.active_connection == self.connections.len()
    }

    fn handle_pending_message(&mut self, msg: &Message) -> bool {
        let pending = match self.pending_connection {
            Some(p) => p,
            None => return false,
        };
        match msg {
            Message::Up => {
                match pending {
                    PendingScreen::PortSelect => {
                        if self.selected_port_index > 0 {
                            self.selected_port_index -= 1;
                        }
                    }
                    PendingScreen::BaudSelect => {
                        if self.selected_baud_index > 0 {
                            self.selected_baud_index -= 1;
                        }
                    }
                    PendingScreen::DataBitsSelect => {
                        if self.selected_data_bits_index > 0 {
                            self.selected_data_bits_index -= 1;
                        }
                    }
                    PendingScreen::ParitySelect => {
                        if self.selected_parity_index > 0 {
                            self.selected_parity_index -= 1;
                        }
                    }
                    PendingScreen::StopBitsSelect => {
                        if self.selected_stop_bits_index > 0 {
                            self.selected_stop_bits_index -= 1;
                        }
                    }
                }
                true
            }
            Message::Down => {
                match pending {
                    PendingScreen::PortSelect => {
                        if !self.available_ports.is_empty()
                            && self.selected_port_index < self.available_ports.len() - 1
                        {
                            self.selected_port_index += 1;
                        }
                    }
                    PendingScreen::BaudSelect => {
                        if self.selected_baud_index < BAUD_RATES.len() - 1 {
                            self.selected_baud_index += 1;
                        }
                    }
                    PendingScreen::DataBitsSelect => {
                        if self.selected_data_bits_index < DATA_BITS_OPTIONS.len() - 1 {
                            self.selected_data_bits_index += 1;
                        }
                    }
                    PendingScreen::ParitySelect => {
                        if self.selected_parity_index < PARITY_OPTIONS.len() - 1 {
                            self.selected_parity_index += 1;
                        }
                    }
                    PendingScreen::StopBitsSelect => {
                        if self.selected_stop_bits_index < STOP_BITS_OPTIONS.len() - 1 {
                            self.selected_stop_bits_index += 1;
                        }
                    }
                }
                true
            }
            Message::Select => {
                match pending {
                    PendingScreen::PortSelect => {
                        if !self.available_ports.is_empty() {
                            self.pending_connection = Some(PendingScreen::BaudSelect);
                        }
                    }
                    PendingScreen::BaudSelect => {
                        self.pending_connection = Some(PendingScreen::DataBitsSelect);
                    }
                    PendingScreen::DataBitsSelect => {
                        self.pending_connection = Some(PendingScreen::ParitySelect);
                    }
                    PendingScreen::ParitySelect => {
                        self.pending_connection = Some(PendingScreen::StopBitsSelect);
                    }
                    PendingScreen::StopBitsSelect => {
                        self.connect_selected();
                    }
                }
                true
            }
            Message::Back => {
                match pending {
                    PendingScreen::PortSelect => {
                        self.pending_connection = None;
                        if !self.connections.is_empty() {
                            self.active_connection = self.connections.len() - 1;
                        }
                    }
                    PendingScreen::BaudSelect => {
                        self.pending_connection = Some(PendingScreen::PortSelect);
                    }
                    PendingScreen::DataBitsSelect => {
                        self.pending_connection = Some(PendingScreen::BaudSelect);
                    }
                    PendingScreen::ParitySelect => {
                        self.pending_connection = Some(PendingScreen::DataBitsSelect);
                    }
                    PendingScreen::StopBitsSelect => {
                        self.pending_connection = Some(PendingScreen::ParitySelect);
                    }
                }
                true
            }
            Message::RefreshPorts => {
                self.refresh_ports();
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, msg: Message) {
        if self.is_pending_active() && self.handle_pending_message(&msg) {
            return;
        }
        match msg {
            Message::Quit => {
                if self.connections.is_empty() {
                    self.should_quit = true;
                } else {
                    self.dialog = Some(Dialog::ConfirmQuit);
                }
            }

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
                Screen::DataBitsSelect => {
                    if self.selected_data_bits_index > 0 {
                        self.selected_data_bits_index -= 1;
                    }
                }
                Screen::ParitySelect => {
                    if self.selected_parity_index > 0 {
                        self.selected_parity_index -= 1;
                    }
                }
                Screen::StopBitsSelect => {
                    if self.selected_stop_bits_index > 0 {
                        self.selected_stop_bits_index -= 1;
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
                Screen::DataBitsSelect => {
                    if self.selected_data_bits_index < DATA_BITS_OPTIONS.len() - 1 {
                        self.selected_data_bits_index += 1;
                    }
                }
                Screen::ParitySelect => {
                    if self.selected_parity_index < PARITY_OPTIONS.len() - 1 {
                        self.selected_parity_index += 1;
                    }
                }
                Screen::StopBitsSelect => {
                    if self.selected_stop_bits_index < STOP_BITS_OPTIONS.len() - 1 {
                        self.selected_stop_bits_index += 1;
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
                    self.screen = Screen::DataBitsSelect;
                }
                Screen::DataBitsSelect => {
                    self.screen = Screen::ParitySelect;
                }
                Screen::ParitySelect => {
                    self.screen = Screen::StopBitsSelect;
                }
                Screen::StopBitsSelect => {
                    self.connect_selected();
                }
                _ => {}
            },

            Message::Back => match self.screen {
                Screen::PortSelect => {
                    if self.connections.is_empty() {
                        self.should_quit = true;
                    }
                }
                Screen::BaudSelect => {
                    self.screen = Screen::PortSelect;
                }
                Screen::DataBitsSelect => {
                    self.screen = Screen::BaudSelect;
                }
                Screen::ParitySelect => {
                    self.screen = Screen::DataBitsSelect;
                }
                Screen::StopBitsSelect => {
                    self.screen = Screen::ParitySelect;
                }
                _ => {}
            },

            Message::RefreshPorts => {
                self.refresh_ports();
            }

            Message::NewConnection => {
                if self.screen == Screen::Connected && self.pending_connection.is_none() {
                    self.pending_connection = Some(PendingScreen::PortSelect);
                    self.refresh_ports();
                    self.active_connection = self.connections.len();
                }
            }

            Message::CloseConnection => {
                if !self.connections.is_empty() && self.active_connection < self.connections.len() {
                    self.dialog = Some(Dialog::ConfirmCloseConnection);
                }
            }

            Message::NextTab => {
                let total = self.connections.len()
                    + if self.pending_connection.is_some() {
                        1
                    } else {
                        0
                    };
                if total > 0 {
                    self.active_connection = (self.active_connection + 1) % total;
                }
            }

            Message::PrevTab => {
                let total = self.connections.len()
                    + if self.pending_connection.is_some() {
                        1
                    } else {
                        0
                    };
                if total > 0 {
                    self.active_connection = if self.active_connection == 0 {
                        total - 1
                    } else {
                        self.active_connection - 1
                    };
                }
            }

            Message::SwitchTab(n) => {
                let total = self.connections.len()
                    + if self.pending_connection.is_some() {
                        1
                    } else {
                        0
                    };
                if n < total {
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
                if !self.input_buffer.is_empty()
                    && !self.connections.is_empty()
                    && self.active_connection < self.connections.len()
                {
                    let data = format!("{}\r\n", self.input_buffer);
                    self.connections[self.active_connection].send(data.as_bytes());
                    self.input_buffer.clear();
                }
            }

            Message::ExportScrollback => {
                if !self.connections.is_empty() && self.active_connection < self.connections.len() {
                    let filename = self.generate_filename(self.active_connection);
                    let cursor_pos = filename.len();
                    self.dialog = Some(Dialog::FileNamePrompt {
                        connection_idx: self.active_connection,
                        filename,
                        cursor_pos,
                        after: AfterSave::Nothing,
                    });
                }
            }

            Message::ScrollUp => {
                if !self.connections.is_empty() && self.active_connection < self.connections.len() {
                    let conn = &mut self.connections[self.active_connection];
                    let total = conn.scrollback.len();
                    conn.scroll_offset = (conn.scroll_offset + 5).min(total);
                }
            }

            Message::ScrollDown => {
                if !self.connections.is_empty() && self.active_connection < self.connections.len() {
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

            Message::DialogYes => {
                self.handle_dialog_yes();
            }

            Message::DialogNo => {
                self.handle_dialog_no();
            }

            Message::DialogCancel => {
                self.dialog = None;
            }

            Message::DialogConfirm => {
                self.handle_dialog_confirm();
            }

            Message::DialogCharInput(c) => {
                if let Some(Dialog::FileNamePrompt {
                    filename,
                    cursor_pos,
                    ..
                }) = &mut self.dialog
                {
                    filename.insert(*cursor_pos, c);
                    *cursor_pos += 1;
                }
            }

            Message::DialogBackspace => {
                if let Some(Dialog::FileNamePrompt {
                    filename,
                    cursor_pos,
                    ..
                }) = &mut self.dialog
                {
                    if *cursor_pos > 0 {
                        filename.remove(*cursor_pos - 1);
                        *cursor_pos -= 1;
                    }
                }
            }

            Message::DialogCursorLeft => {
                if let Some(Dialog::FileNamePrompt { cursor_pos, .. }) = &mut self.dialog {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                    }
                }
            }

            Message::DialogCursorRight => {
                if let Some(Dialog::FileNamePrompt {
                    filename,
                    cursor_pos,
                    ..
                }) = &mut self.dialog
                {
                    if *cursor_pos < filename.len() {
                        *cursor_pos += 1;
                    }
                }
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
            // No menu open — check for content area clicks
            self.handle_content_click(col, row);
            return;
        };

        let drop_w = 0..16_u16; // dropdown is 16 chars wide
        let handled = match menu {
            OpenMenu::File => {
                let drop_col = col.wrapping_sub(MENU_FILE_X);
                if row == 2 && drop_w.contains(&drop_col) {
                    // Export
                    self.open_menu = None;
                    if !self.connections.is_empty() {
                        let filename = self.generate_filename(self.active_connection);
                        let cursor_pos = filename.len();
                        self.dialog = Some(Dialog::FileNamePrompt {
                            connection_idx: self.active_connection,
                            filename,
                            cursor_pos,
                            after: AfterSave::Nothing,
                        });
                    }
                    true
                } else if row == 3 && drop_w.contains(&drop_col) {
                    // Quit
                    self.open_menu = None;
                    if self.connections.is_empty() {
                        self.should_quit = true;
                    } else {
                        self.dialog = Some(Dialog::ConfirmQuit);
                    }
                    true
                } else {
                    false
                }
            }
            OpenMenu::Connection => {
                let drop_col = col.wrapping_sub(MENU_CONN_X);
                if row == 2 && drop_w.contains(&drop_col) {
                    self.open_menu = None;
                    if self.screen == Screen::Connected && self.pending_connection.is_none() {
                        self.pending_connection = Some(PendingScreen::PortSelect);
                        self.refresh_ports();
                        self.active_connection = self.connections.len();
                    }
                    true
                } else if row == 3 && drop_w.contains(&drop_col) {
                    // Close
                    self.open_menu = None;
                    if !self.connections.is_empty() {
                        self.dialog = Some(Dialog::ConfirmCloseConnection);
                    }
                    true
                } else {
                    false
                }
            }
            OpenMenu::View => {
                let drop_col = col.wrapping_sub(MENU_VIEW_X);
                if row == 2 && drop_w.contains(&drop_col) {
                    self.open_menu = None;
                    self.view_mode = ViewMode::Tabs;
                    true
                } else if row == 3 && drop_w.contains(&drop_col) {
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

    fn handle_content_click(&mut self, col: u16, row: u16) {
        match self.screen {
            Screen::PortSelect => {
                // Layout: row 0 = menu bar, row 1 = top border, rows 2+ = items,
                // bottom = bottom border + status bar
                let inner_top = 2_u16;
                let inner_bottom = self.terminal_rows.saturating_sub(2); // status(1) + border(1)
                if row >= inner_top && row < inner_bottom {
                    let visible_height = (inner_bottom - inner_top) as usize;
                    let visual_row = (row - inner_top) as usize;
                    let count = self.available_ports.len();
                    let offset =
                        list_scroll_offset(self.selected_port_index, visible_height, count);
                    let item_index = offset + visual_row;
                    if item_index < count {
                        self.selected_port_index = item_index;
                        self.screen = Screen::BaudSelect;
                    }
                }
            }
            Screen::BaudSelect => {
                let inner_top = 2_u16;
                let inner_bottom = self.terminal_rows.saturating_sub(2);
                if row >= inner_top && row < inner_bottom {
                    let visible_height = (inner_bottom - inner_top) as usize;
                    let visual_row = (row - inner_top) as usize;
                    let count = BAUD_RATES.len();
                    let offset =
                        list_scroll_offset(self.selected_baud_index, visible_height, count);
                    let item_index = offset + visual_row;
                    if item_index < count {
                        self.selected_baud_index = item_index;
                        self.screen = Screen::DataBitsSelect;
                    }
                }
            }
            Screen::DataBitsSelect => {
                let inner_top = 2_u16;
                let inner_bottom = self.terminal_rows.saturating_sub(2);
                if row >= inner_top && row < inner_bottom {
                    let visible_height = (inner_bottom - inner_top) as usize;
                    let visual_row = (row - inner_top) as usize;
                    let count = DATA_BITS_OPTIONS.len();
                    let offset =
                        list_scroll_offset(self.selected_data_bits_index, visible_height, count);
                    let item_index = offset + visual_row;
                    if item_index < count {
                        self.selected_data_bits_index = item_index;
                        self.screen = Screen::ParitySelect;
                    }
                }
            }
            Screen::ParitySelect => {
                let inner_top = 2_u16;
                let inner_bottom = self.terminal_rows.saturating_sub(2);
                if row >= inner_top && row < inner_bottom {
                    let visible_height = (inner_bottom - inner_top) as usize;
                    let visual_row = (row - inner_top) as usize;
                    let count = PARITY_OPTIONS.len();
                    let offset =
                        list_scroll_offset(self.selected_parity_index, visible_height, count);
                    let item_index = offset + visual_row;
                    if item_index < count {
                        self.selected_parity_index = item_index;
                        self.screen = Screen::StopBitsSelect;
                    }
                }
            }
            Screen::StopBitsSelect => {
                let inner_top = 2_u16;
                let inner_bottom = self.terminal_rows.saturating_sub(2);
                if row >= inner_top && row < inner_bottom {
                    let visible_height = (inner_bottom - inner_top) as usize;
                    let visual_row = (row - inner_top) as usize;
                    let count = STOP_BITS_OPTIONS.len();
                    let offset =
                        list_scroll_offset(self.selected_stop_bits_index, visible_height, count);
                    let item_index = offset + visual_row;
                    if item_index < count {
                        self.selected_stop_bits_index = item_index;
                        self.connect_selected();
                    }
                }
            }
            Screen::Connected => {
                if self.connections.is_empty() && self.pending_connection.is_none() {
                    return;
                }

                // Layout: row 0 = menu bar, row 1+ = content area
                // Content splits into: main_area, input_area(3 rows), status_bar(1 row)
                let content_top = 1_u16;
                let status_and_input = 4_u16;
                let main_bottom = self.terminal_rows.saturating_sub(status_and_input);

                match self.view_mode {
                    ViewMode::Tabs => {
                        if row == content_top {
                            self.handle_tab_bar_click(col);
                        } else if self.is_pending_active() && row > content_top && row < main_bottom
                        {
                            self.handle_pending_click(row, content_top + 1, main_bottom);
                        }
                    }
                    ViewMode::Grid => {
                        if row >= content_top && row < main_bottom {
                            self.handle_grid_click(col, row, content_top, main_bottom);
                        }
                    }
                }
            }
        }
    }

    fn handle_tab_bar_click(&mut self, col: u16) {
        let mut x = 0_u16;
        for (i, conn) in self.connections.iter().enumerate() {
            let label_width = conn.label().len() as u16 + 2; // " label "
            if col >= x && col < x + label_width {
                self.active_connection = i;
                return;
            }
            x += label_width;
        }
        // Check "New" tab if pending
        if self.pending_connection.is_some() {
            let new_label_width = 5_u16; // " New "
            if col >= x && col < x + new_label_width {
                self.active_connection = self.connections.len();
                return;
            }
            x += new_label_width;
        }
        // Check [+] button (only shown when no pending)
        if self.pending_connection.is_none() && col >= x && col < x + 5 {
            self.pending_connection = Some(PendingScreen::PortSelect);
            self.refresh_ports();
            self.active_connection = self.connections.len();
        }
    }

    fn handle_grid_click(&mut self, col: u16, row: u16, grid_top: u16, grid_bottom: u16) {
        let total = self.connections.len()
            + if self.pending_connection.is_some() {
                1
            } else {
                0
            };
        if total == 0 {
            return;
        }

        let grid_height = grid_bottom - grid_top;
        let grid_width = self.terminal_cols;

        let grid_cols = (total as f64).sqrt().ceil() as usize;
        let grid_rows = total.div_ceil(grid_cols);

        let cell_h = grid_height as usize / grid_rows;
        let cell_w = grid_width as usize / grid_cols;

        if cell_h == 0 || cell_w == 0 {
            return;
        }

        let r = (row - grid_top) as usize / cell_h;
        let c = col as usize / cell_w;
        let idx = r * grid_cols + c;

        if idx < self.connections.len() {
            self.active_connection = idx;
        } else if idx == self.connections.len() && self.pending_connection.is_some() {
            self.active_connection = self.connections.len();
            let cell_top = grid_top + (r as u16) * (cell_h as u16);
            let cell_bottom = cell_top + cell_h as u16;
            self.handle_pending_click(row, cell_top, cell_bottom);
        }
    }

    fn handle_pending_click(&mut self, row: u16, cell_top: u16, cell_bottom: u16) {
        // Cell has Block with Borders::ALL — inner content is 1 row inside each edge
        let inner_top = cell_top + 1;
        let inner_bottom = cell_bottom.saturating_sub(1);
        if row < inner_top || row >= inner_bottom {
            return;
        }

        let visible_height = (inner_bottom - inner_top) as usize;
        let visual_row = (row - inner_top) as usize;

        match self.pending_connection {
            Some(PendingScreen::PortSelect) => {
                let count = self.available_ports.len();
                let offset = list_scroll_offset(self.selected_port_index, visible_height, count);
                let item_index = offset + visual_row;
                if item_index < count {
                    self.selected_port_index = item_index;
                    self.pending_connection = Some(PendingScreen::BaudSelect);
                }
            }
            Some(PendingScreen::BaudSelect) => {
                let count = BAUD_RATES.len();
                let offset = list_scroll_offset(self.selected_baud_index, visible_height, count);
                let item_index = offset + visual_row;
                if item_index < count {
                    self.selected_baud_index = item_index;
                    self.pending_connection = Some(PendingScreen::DataBitsSelect);
                }
            }
            Some(PendingScreen::DataBitsSelect) => {
                let count = DATA_BITS_OPTIONS.len();
                let offset =
                    list_scroll_offset(self.selected_data_bits_index, visible_height, count);
                let item_index = offset + visual_row;
                if item_index < count {
                    self.selected_data_bits_index = item_index;
                    self.pending_connection = Some(PendingScreen::ParitySelect);
                }
            }
            Some(PendingScreen::ParitySelect) => {
                let count = PARITY_OPTIONS.len();
                let offset =
                    list_scroll_offset(self.selected_parity_index, visible_height, count);
                let item_index = offset + visual_row;
                if item_index < count {
                    self.selected_parity_index = item_index;
                    self.pending_connection = Some(PendingScreen::StopBitsSelect);
                }
            }
            Some(PendingScreen::StopBitsSelect) => {
                let count = STOP_BITS_OPTIONS.len();
                let offset =
                    list_scroll_offset(self.selected_stop_bits_index, visible_height, count);
                let item_index = offset + visual_row;
                if item_index < count {
                    self.selected_stop_bits_index = item_index;
                    self.connect_selected();
                }
            }
            None => {}
        }
    }

    fn handle_dialog_yes(&mut self) {
        match self.dialog.take() {
            Some(Dialog::ConfirmCloseConnection) => {
                let idx = self.active_connection;
                let filename = self.generate_filename(idx);
                let cursor_pos = filename.len();
                self.dialog = Some(Dialog::FileNamePrompt {
                    connection_idx: idx,
                    filename,
                    cursor_pos,
                    after: AfterSave::CloseConnection,
                });
            }
            Some(Dialog::ConfirmQuit) => {
                let indices: Vec<usize> = (0..self.connections.len()).collect();
                self.start_save_chain(indices);
            }
            _ => {}
        }
    }

    fn handle_dialog_no(&mut self) {
        match self.dialog.take() {
            Some(Dialog::ConfirmCloseConnection) => {
                self.do_close_active_connection();
            }
            Some(Dialog::ConfirmQuit) => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn handle_dialog_confirm(&mut self) {
        if let Some(Dialog::FileNamePrompt {
            connection_idx,
            filename,
            after,
            ..
        }) = self.dialog.take()
        {
            self.export_connection(connection_idx, &filename);
            match after {
                AfterSave::Nothing => {}
                AfterSave::CloseConnection => {
                    self.do_close_active_connection();
                }
                AfterSave::QuitNext { remaining } => {
                    self.start_save_chain(remaining);
                }
            }
        }
    }

    fn start_save_chain(&mut self, mut indices: Vec<usize>) {
        if let Some(idx) = indices.first().copied() {
            indices.remove(0);
            let filename = self.generate_filename(idx);
            let cursor_pos = filename.len();
            self.dialog = Some(Dialog::FileNamePrompt {
                connection_idx: idx,
                filename,
                cursor_pos,
                after: AfterSave::QuitNext { remaining: indices },
            });
        } else {
            self.should_quit = true;
        }
    }

    fn do_close_active_connection(&mut self) {
        if self.connections.is_empty() {
            return;
        }
        let idx = self.active_connection;
        self.connections[idx].close();
        self.connections.remove(idx);
        if self.connections.is_empty() {
            self.screen = Screen::PortSelect;
            self.pending_connection = None;
            self.refresh_ports();
        } else if self.active_connection >= self.connections.len() {
            self.active_connection = self.connections.len() - 1;
        }
    }

    fn connect_selected(&mut self) {
        if self.available_ports.is_empty() {
            return;
        }
        let port_name = self.available_ports[self.selected_port_index].name.clone();
        let baud_rate = BAUD_RATES[self.selected_baud_index];
        let data_bits = DATA_BITS_OPTIONS[self.selected_data_bits_index].1;
        let parity = PARITY_OPTIONS[self.selected_parity_index].1;
        let stop_bits = STOP_BITS_OPTIONS[self.selected_stop_bits_index].1;
        let id = self.next_connection_id;
        self.next_connection_id += 1;

        let conn = Connection::new(
            id,
            port_name,
            baud_rate,
            data_bits,
            parity,
            stop_bits,
            self.serial_tx.clone(),
        );
        self.connections.push(conn);
        self.active_connection = self.connections.len() - 1;
        self.pending_connection = None;
        self.screen = Screen::Connected;
    }

    fn generate_filename(&self, connection_idx: usize) -> String {
        let conn = &self.connections[connection_idx];
        let safe_name = conn.port_name.replace(['/', '\\', ':'], "_");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        format!("{}_{}_{}.txt", safe_name, conn.baud_rate, timestamp)
    }

    fn export_connection(&mut self, connection_idx: usize, filename: &str) {
        if connection_idx >= self.connections.len() {
            return;
        }
        let conn = &self.connections[connection_idx];
        let content: String = conn
            .scrollback_with_partial()
            .collect::<Vec<_>>()
            .join("\n");

        match std::fs::write(filename, &content) {
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

/// Compute the scroll offset ratatui's List widget uses when `ListState` starts at offset 0.
fn list_scroll_offset(selected: usize, visible_height: usize, _count: usize) -> usize {
    if visible_height == 0 {
        return 0;
    }
    if selected >= visible_height {
        selected - visible_height + 1
    } else {
        0
    }
}
