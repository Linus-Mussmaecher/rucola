/// Messages sent from the user to the application
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    /// Quit the application
    Quit,
    /// Switch to select screen
    SwitchSelect,
}
