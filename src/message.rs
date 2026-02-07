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

    // Export
    ExportScrollback,

    // Scroll
    ScrollUp,
    ScrollDown,

    // Menu
    MenuClick(u16, u16),
    CloseMenu,

    // Dialog responses
    DialogYes,
    DialogNo,
    DialogCancel,
    DialogConfirm,
    DialogCharInput(char),
    DialogBackspace,
}
