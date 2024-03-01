use crossterm::event::KeyCode;

/// A struc that manages a sequence of key inputs.
pub struct SequenceManager {
    sequence: Vec<char>,
}

impl SequenceManager {
    /// Creates a new sequence manager with an empty sequence and 4 space reserved.
    pub fn new() -> Self {
        Self {
            sequence: Vec::with_capacity(4),
        }
    }

    /// Resets the sequence to empty.
    pub fn reset_sequence(&mut self) {
        self.sequence.clear();
    }

    /// Returns the currently saved sequence.
    pub fn get_sequence(&self) -> &[char] {
        &self.sequence
    }

    /// Adds a key code to the sequence if it is a KeyCode::Char and returns wether or not this happened.
    pub fn register(&mut self, key: &KeyCode) -> bool {
        if let KeyCode::Char(c) = key {
            self.sequence.push(*c);
            true
        } else {
            false
        }
    }
}
