use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use super::worker::{self, SerialEvent};

#[derive(Clone, Copy, PartialEq)]
pub enum DisplayMode {
    Text,
    HexDump,
}

pub struct Connection {
    pub id: usize,
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: serialport::DataBits,
    pub parity: serialport::Parity,
    pub stop_bits: serialport::StopBits,
    pub display_mode: DisplayMode,
    pub scrollback: Vec<String>,
    pub scroll_offset: usize,
    pub write_tx: Option<mpsc::Sender<Vec<u8>>>,
    pub alive: bool,
    thread_handle: Option<JoinHandle<()>>,
    line_buffer: String,
    raw_bytes: Vec<u8>,
    hex_bytes_formatted: usize,
}

impl Connection {
    pub fn new(
        id: usize,
        port_name: String,
        baud_rate: u32,
        data_bits: serialport::DataBits,
        parity: serialport::Parity,
        stop_bits: serialport::StopBits,
        display_mode: DisplayMode,
        serial_tx: mpsc::Sender<SerialEvent>,
    ) -> Self {
        let (write_tx, write_rx) = mpsc::channel();
        let name = port_name.clone();

        let handle = thread::spawn(move || {
            worker::connection_thread(
                id, &name, baud_rate, data_bits, parity, stop_bits, serial_tx, write_rx,
            );
        });

        let data_bits_str = match data_bits {
            serialport::DataBits::Five => "5",
            serialport::DataBits::Six => "6",
            serialport::DataBits::Seven => "7",
            serialport::DataBits::Eight => "8",
        };
        let parity_str = match parity {
            serialport::Parity::None => "N",
            serialport::Parity::Odd => "O",
            serialport::Parity::Even => "E",
        };
        let stop_str = match stop_bits {
            serialport::StopBits::One => "1",
            serialport::StopBits::Two => "2",
        };
        let mode_str = match display_mode {
            DisplayMode::Text => "text",
            DisplayMode::HexDump => "hex",
        };
        let start_msg = format!(
            "--- Connected to {} at {} baud ({}{}{}, {}) ---",
            port_name, baud_rate, data_bits_str, parity_str, stop_str, mode_str
        );
        Self {
            id,
            port_name,
            baud_rate,
            data_bits,
            parity,
            stop_bits,
            display_mode,
            scrollback: vec![start_msg],
            scroll_offset: 0,
            write_tx: Some(write_tx),
            alive: true,
            thread_handle: Some(handle),
            line_buffer: String::new(),
            raw_bytes: Vec::new(),
            hex_bytes_formatted: 0,
        }
    }

    pub fn label(&self) -> String {
        let data_bits_ch = match self.data_bits {
            serialport::DataBits::Five => '5',
            serialport::DataBits::Six => '6',
            serialport::DataBits::Seven => '7',
            serialport::DataBits::Eight => '8',
        };
        let parity_ch = match self.parity {
            serialport::Parity::None => 'N',
            serialport::Parity::Odd => 'O',
            serialport::Parity::Even => 'E',
        };
        let stop_ch = match self.stop_bits {
            serialport::StopBits::One => '1',
            serialport::StopBits::Two => '2',
        };
        let suffix = match self.display_mode {
            DisplayMode::HexDump => " HEX",
            DisplayMode::Text => "",
        };
        format!(
            "{}@{}/{}{}{}{}",
            self.port_name, self.baud_rate, data_bits_ch, parity_ch, stop_ch, suffix
        )
    }

    pub fn push_data(&mut self, data: &[u8]) {
        match self.display_mode {
            DisplayMode::Text => {
                let text = String::from_utf8_lossy(data);
                for ch in text.chars() {
                    if ch == '\n' {
                        self.scrollback.push(std::mem::take(&mut self.line_buffer));
                    } else if ch != '\r' {
                        self.line_buffer.push(ch);
                    }
                }
            }
            DisplayMode::HexDump => {
                self.raw_bytes.extend_from_slice(data);
                // Format complete 16-byte rows into scrollback
                let complete_rows = self.raw_bytes.len() / 16;
                let already_done = self.hex_bytes_formatted / 16;
                for row in already_done..complete_rows {
                    let offset = row * 16;
                    let line = format_hex_line(offset, &self.raw_bytes[offset..offset + 16]);
                    self.scrollback.push(line);
                }
                self.hex_bytes_formatted = complete_rows * 16;
                // Update line_buffer with partial row (so scrollback_with_partial works)
                let remaining = &self.raw_bytes[self.hex_bytes_formatted..];
                if remaining.is_empty() {
                    self.line_buffer.clear();
                } else {
                    self.line_buffer = format_hex_line(self.hex_bytes_formatted, remaining);
                }
            }
        }
    }

    pub fn send(&self, data: &[u8]) {
        if let Some(tx) = &self.write_tx {
            let _ = tx.send(data.to_vec());
        }
    }

    pub fn close(&mut self) {
        self.write_tx.take(); // drop sender to signal thread
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        self.alive = false;
    }

    pub fn scrollback_with_partial(&self) -> impl Iterator<Item = &str> {
        self.scrollback
            .iter()
            .map(|s| s.as_str())
            .chain(if self.line_buffer.is_empty() {
                None
            } else {
                Some(self.line_buffer.as_str())
            })
    }
}

fn format_hex_line(offset: usize, bytes: &[u8]) -> String {
    let mut hex_part = String::with_capacity(49);
    for (i, &b) in bytes.iter().enumerate() {
        if i == 8 {
            hex_part.push(' ');
        }
        if i > 0 {
            hex_part.push(' ');
        }
        hex_part.push_str(&format!("{:02X}", b));
    }
    // Pad hex section to full width (16 bytes = "XX XX XX XX XX XX XX XX  XX XX XX XX XX XX XX XX")
    let full_hex_width = 48; // 16*3 - 1 + 1 (extra space between groups)
    while hex_part.len() < full_hex_width {
        hex_part.push(' ');
    }

    let ascii: String = bytes
        .iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
        .collect();

    format!("{:08X}  {}  |{}|", offset, hex_part, ascii)
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.close();
    }
}
