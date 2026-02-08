# serialtui

A terminal-based serial port monitor with support for multiple simultaneous connections. Built with Rust and [ratatui](https://ratatui.rs).

## Features

- **Port discovery** — lists all available serial ports with descriptions
- **Configurable baud rate** — 300 to 921600, defaults to 9600
- **Bidirectional communication** — read from and write to serial ports
- **Multiple connections** — open several ports at once, switch between them
- **Tab and grid views** — view one connection at a time or all at once in a split layout
- **Unlimited scrollback** per connection with arrow keys, PageUp/PageDown, and mouse wheel scrolling
- **Export to file** — save scrollback as `.txt` with editable filename prompt (`Ctrl+E` or File menu)
- **Save on close/quit** — prompted to export sessions when closing a connection or quitting
- **Clickable UI** — menu bar (File, Connection, View), clickable tabs, clickable grid cells, and mouse support
- **Connection banner** — each session starts with a `--- Connected to <port> at <baud> baud ---` line
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

Open additional connections with `Ctrl+N` or click the green `[+]` tab — a "New" tab appears inline where you can select port and baud rate without leaving the connected view.

### Exporting

When exporting (via `Ctrl+E`, the File menu, or when closing/quitting), a filename prompt appears pre-filled with a generated name in the format:

```
<port>_<baud>_YYYYMMDD_HHMMSS.txt
```

Press Enter to accept, edit the name, or Esc to cancel.

When closing a connection (`Ctrl+W`) or quitting (`Ctrl+Q`), you are asked whether to save the session first. Choosing "Yes" walks through a filename prompt for each connection.

### Key Bindings

#### Port Selection (initial)
| Key | Action |
|-----|--------|
| Up/Down | Navigate |
| Enter | Select port |
| r | Refresh port list |
| Esc / q | Quit |

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
| Ctrl+N | New connection (inline tab) |
| Ctrl+W | Close active connection (prompts to save) |
| Ctrl+E | Export scrollback to .txt |
| Ctrl+G | Toggle tab / grid view |
| Up / Down | Scroll line by line |
| PageUp / PageDown | Scroll |
| Mouse wheel | Scroll |
| Mouse click | Switch tab or grid cell |
| Enter | Send input |
| Ctrl+Q | Quit (prompts to save all) |

## Building

Requires Rust 1.70+.

```
cargo build --release
```

The release binary is at `target/release/serialtui` (or `serialtui.exe` on Windows).

## License

MIT
