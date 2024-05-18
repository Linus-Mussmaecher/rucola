/// Messages sent from the user to the application
#[derive(Debug)]
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
