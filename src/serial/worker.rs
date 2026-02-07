use std::io::Read;
use std::sync::mpsc;
use std::time::Duration;

pub enum SerialEvent {
    Data { id: usize, data: Vec<u8> },
    Error { id: usize, err: String },
    Disconnected { id: usize },
}

pub fn connection_thread(
    id: usize,
    port_name: &str,
    baud_rate: u32,
    serial_tx: mpsc::Sender<SerialEvent>,
    write_rx: mpsc::Receiver<Vec<u8>>,
) {
    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    let mut port = match port {
        Ok(p) => p,
        Err(e) => {
            let _ = serial_tx.send(SerialEvent::Error {
                id,
                err: e.to_string(),
            });
            return;
        }
    };

    let mut buf = [0u8; 1024];

    loop {
        // Check for data to write
        match write_rx.try_recv() {
            Ok(data) => {
                use std::io::Write;
                if let Err(e) = port.write_all(&data) {
                    let _ = serial_tx.send(SerialEvent::Error {
                        id,
                        err: e.to_string(),
                    });
                    break;
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // Main thread dropped write_tx — time to exit
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        // Read from port
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                let _ = serial_tx.send(SerialEvent::Data {
                    id,
                    data: buf[..n].to_vec(),
                });
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                let _ = serial_tx.send(SerialEvent::Error {
                    id,
                    err: e.to_string(),
                });
                break;
            }
        }
    }

    let _ = serial_tx.send(SerialEvent::Disconnected { id });
}
