pub enum Message {
    // Navigation
    Quit,
    Up,
    Down,
    Select,
    Back,

    // Ports
    RefreshPorts,

    // Connections
    NewConnection,
    CloseConnection,
    NextTab,
    PrevTab,
    SwitchTab(usize),

    // View
    ToggleViewMode,

    // Input
    CharInput(char),
    Backspace,
    SendInput,

    // Scroll
    ScrollUp,
    ScrollDown,
}
