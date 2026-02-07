use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use super::worker::{self, SerialEvent};

pub struct Connection {
    pub id: usize,
    pub port_name: String,
    pub baud_rate: u32,
    pub scrollback: Vec<String>,
    pub scroll_offset: usize,
    pub write_tx: Option<mpsc::Sender<Vec<u8>>>,
    pub alive: bool,
    thread_handle: Option<JoinHandle<()>>,
    line_buffer: String,
}

impl Connection {
    pub fn new(
        id: usize,
        port_name: String,
        baud_rate: u32,
        serial_tx: mpsc::Sender<SerialEvent>,
    ) -> Self {
        let (write_tx, write_rx) = mpsc::channel();
        let name = port_name.clone();

        let handle = thread::spawn(move || {
            worker::connection_thread(id, &name, baud_rate, serial_tx, write_rx);
        });

        let start_msg = format!("--- Connected to {} at {} baud ---", port_name, baud_rate);
        Self {
            id,
            port_name,
            baud_rate,
            scrollback: vec![start_msg],
            scroll_offset: 0,
            write_tx: Some(write_tx),
            alive: true,
            thread_handle: Some(handle),
            line_buffer: String::new(),
        }
    }

    pub fn label(&self) -> String {
        format!("{}@{}", self.port_name, self.baud_rate)
    }

    pub fn push_data(&mut self, data: &[u8]) {
        let text = String::from_utf8_lossy(data);
        for ch in text.chars() {
            if ch == '\n' {
                self.scrollback.push(std::mem::take(&mut self.line_buffer));
            } else if ch != '\r' {
                self.line_buffer.push(ch);
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

impl Drop for Connection {
    fn drop(&mut self) {
        self.close();
    }
}
