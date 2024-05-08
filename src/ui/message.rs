/// Messages sent from the user to the application
#[derive(Debug)]
pub enum Message {
    /// Quit the application
    Quit,
    /// Switch to select screen
    SwitchSelect { refresh: bool },
    /// Switches to displaying the (markdown) file at the given path
    SwitchDisplay(String),
    /// Restore the terminal, execute the given command and re-enter
    OpenExternalCommand(std::process::Command),
}
