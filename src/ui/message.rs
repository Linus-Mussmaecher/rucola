/// Messages sent from the user to the application
#[derive(Debug)]
pub enum Message {
    /// Quit the application
    Quit,
    ///
    DisplayStackClear,
    ///
    DisplayStackPop,
    ///
    DisplayStackPush(String),
    /// Restore the terminal, execute the given command and re-enter
    OpenExternalCommand(std::process::Command),
}
