# serialtui

TUI serial port terminal built with Rust + ratatui. Targets Windows as the primary release platform.

## Build

```
cargo check && cargo build
```

Release builds use `--release` with LTO and symbol stripping (see `Cargo.toml` `[profile.release]`).

## Architecture

TEA (The Elm Architecture) main loop in `src/main.rs`:
1. `terminal.draw()` — renders UI based on `App` state
2. `input::poll_event()` — polls crossterm events (50ms timeout), maps to `Message`
3. `app.drain_serial_events()` — drains mpsc channel from serial worker threads
4. `app.update(msg)` — mutates state

### Serial I/O

One `std::thread` per connection. Each thread opens a serial port with 10ms read timeout, reads into a buffer, and checks a write channel for outbound data. Communication with the main thread uses `std::sync::mpsc`:
- `serial_tx` (shared) — worker threads send `SerialEvent` to main thread
- `write_tx` (per connection) — main thread sends data to worker thread
- Dropping `write_tx` signals the worker to exit

### Module Layout

- `src/app.rs` — `App` state, `Screen`/`ViewMode` enums, `update()` dispatch
- `src/message.rs` — `Message` enum for all user input events
- `src/input.rs` — crossterm event → `Message` mapping, keybindings per screen
- `src/serial/connection.rs` — `Connection` struct (scrollback, channels, thread handle)
- `src/serial/worker.rs` — `connection_thread()` serial read/write loop, `SerialEvent` enum
- `src/ui/` — all rendering: `port_select`, `baud_select`, `terminal_view`, `status_bar`

## CI/CD

GitHub Actions workflow at `.github/workflows/release.yml`:
- Triggers on `v*` tags
- Builds on Blacksmith CI (`blacksmith-2vcpu-windows-2025`)
- Cross-compiles for `x86_64-pc-windows-msvc`
- Attaches `serialtui.exe` to a GitHub release

## Before Committing

Per global instructions:
```
cargo check && cargo build
```
Only commit if both succeed.
