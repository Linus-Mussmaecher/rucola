/// Opening mode for the OpenNote messages
#[derive(Copy, PartialEq, Eq, Clone, Debug, Default)]
pub enum OpeningMode {
    #[default]
    /// Open the note's html for passive viewing.
    VIEW,
    /// Open the note for active editing.
    EDIT,
}

/// Messages sent from the user to the application
#[derive(Debug, Clone)]
pub enum Message {
    /// No message
    None,
    /// Quit the application
    Quit,
    /// Fully refresh the file index
    Refresh,
    ///
    DisplayStackClear,
    ///
    DisplayStackPop,
    ///
    DisplayStackPush(String),
    /// Open a note
    OpenNote(OpeningMode, std::path::PathBuf),
}

/// Messages sent from the application to the terminal.
#[derive(Debug, Clone)]
pub enum TerminalMessage {
    /// No message
    None,
    /// Quit the application
    Quit,
    /// Open a note
    OpenNote(OpeningMode, std::path::PathBuf),
}

impl From<Message> for TerminalMessage {
    fn from(value: Message) -> Self {
        match value {
            Message::None
            | Message::Refresh
            | Message::DisplayStackClear
            | Message::DisplayStackPop
            | Message::DisplayStackPush(_) => Self::None,
            Message::Quit => Self::Quit,
            Message::OpenNote(mode, path) => Self::OpenNote(mode, path),
        }
    }
}
