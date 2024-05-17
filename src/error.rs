use ratatui::{style, widgets::*};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RucolaError {
    #[error("An IO operation failed: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to find this note at the expected location: {0}.")]
    NoteNoteFound(String),
    #[error("Failed to load config file, defaulting: {0}")]
    ConfigLoad(#[from] confy::ConfyError),
    #[error("Could not find a default application for this file type.")]
    EditorMissing,
}

impl RucolaError {
    pub fn to_ratatui(&self) -> Paragraph<'_> {
        Paragraph::new(format!("{}", &self)).style(style::Style::new().fg(style::Color::Red))
    }
}
