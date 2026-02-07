# serialtui

A terminal-based serial port monitor with support for multiple simultaneous connections. Built with Rust and [ratatui](https://ratatui.rs).

## Features

- **Port discovery** — lists all available serial ports with descriptions
- **Configurable baud rate** — 300 to 921600, defaults to 9600
- **Bidirectional communication** — read from and write to serial ports
- **Multiple connections** — open several ports at once, switch between them
- **Tab and grid views** — view one connection at a time or all at once in a split layout
- **10,000-line scrollback** per connection with PageUp/PageDown scrolling
- **Cross-platform** — runs on Windows, macOS, and Linux (Windows `.exe` provided in releases)

## Installation

Download `serialtui.exe` from the [latest release](https://github.com/dorner-it/serialtui/releases/latest).

Or build from source:

```
cargo build --release
```

## Usage

Run the binary — no arguments needed:

```
serialtui
```

### Workflow

1. **Select a port** from the detected list
2. **Choose a baud rate** (arrow keys + Enter)
3. **Interact** — received data appears in the scrollback, type and press Enter to send

Open additional connections with `Ctrl+N`, which returns you to port selection.

### Key Bindings

#### Port Selection
| Key | Action |
|-----|--------|
| Up/Down | Navigate |
| Enter | Select port |
| r | Refresh port list |
| Esc | Back (if adding connection) |
| q | Quit |

#### Baud Rate Selection
| Key | Action |
|-----|--------|
| Up/Down | Navigate |
| Enter | Connect |
| Esc | Back |

#### Connected View
| Key | Action |
|-----|--------|
| Tab / Shift+Tab | Next / previous connection |
| 1–9 | Jump to connection N |
| Ctrl+N | New connection |
| Ctrl+W | Close active connection |
| Ctrl+G | Toggle tab / grid view |
| PageUp / PageDown | Scroll |
| Enter | Send input |
| Ctrl+Q | Quit |

## Building

Requires Rust 1.70+.

```
cargo build --release
```

The release binary is at `target/release/serialtui` (or `serialtui.exe` on Windows).

## License

MIT
