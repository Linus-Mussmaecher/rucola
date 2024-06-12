/// Messages sent from the user to the application
#[derive(Debug)]
pub enum Message {
    /// No message
    None,
    /// Quit the application
    Quit,
    /// Clears the display stack, returning to the select screen.
    DisplayStackClear,
    /// Pops the top of the display stack, going one page back.
    DisplayStackPop,
    /// Pushes the given id to the top of the display stack.
    DisplayStackPush(String),
    /// Restore the terminal, execute the given command and re-enter
    OpenExternalCommand(std::process::Command),
}

/// Messages sent from the application to the terminal.
#[derive(Debug)]
pub enum TerminalMessage {
    /// No message
    None,
    /// Quit the application
    Quit,
    /// Restore the terminal, execute the given command and re-enter
    OpenExternalCommand(std::process::Command),
}

impl From<Message> for TerminalMessage {
    fn from(value: Message) -> Self {
        match value {
            Message::None
            | Message::DisplayStackClear
            | Message::DisplayStackPop
            | Message::DisplayStackPush(_) => Self::None,
            Message::Quit => Self::Quit,
            Message::OpenExternalCommand(cmd) => Self::OpenExternalCommand(cmd),
        }
    }
}
