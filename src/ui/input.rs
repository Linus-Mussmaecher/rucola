use std::collections::HashMap;

use crossterm::event::KeyCode;

/// A struc that manages a sequence of key inputs.
#[derive(Clone, Debug)]
pub struct InputManager {
    sequence: String,
    key_bindings: HashMap<String, Message>,
}

impl InputManager {
    /// Creates a new sequence manager with an empty sequence and 4 space reserved.
    pub fn new(key_bindings: &[(String, Message)]) -> Self {
        Self {
            sequence: String::with_capacity(4),
            key_bindings: key_bindings.iter().cloned().collect(),
        }
    }

    /// Adds a key code to the sequence if it is a KeyCode::Char and returns any triggered messages.
    pub fn register(&mut self, key: KeyCode) -> Option<Message> {
        if let KeyCode::Char(a) = key {
            self.sequence.push(a);
            if let Some(msg) = self.key_bindings.get(&self.sequence) {
                self.sequence.clear();
                return Some(*msg);
            }
        }
        if key == KeyCode::Esc {
            self.sequence.clear();
        }
        None
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new(&[
            ("q".to_owned(), Message::Quit),
            ("ge".to_owned(), Message::GotoEnd),
            ("d".to_owned(), Message::Clear),
        ])
    }
}

/// Messages sent from the user to the application
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message {
    /// Quit the application
    Quit,
    /// Move the view to the end of the screen
    GotoEnd,
    /// Clear current view.
    Clear,
}
