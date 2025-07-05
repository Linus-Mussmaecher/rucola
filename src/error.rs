use ratatui::{style, widgets::*};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RucolaError>;

#[derive(Error, Debug)]
pub enum RucolaError {
    #[error("An IO operation failed: {0}")]
    IO(#[from] std::io::Error),
    #[error("Failed to find this note at the expected location: {0}.")]
    NoteNotFound(String),
    #[error("Could not read file name of note at {0}.")]
    NoteNameCannotBeRead(std::path::PathBuf),
    #[error("Failed to load config file, defaulting: {0}")]
    ConfigLoad(#[from] confy::ConfyError),
    #[error("Could not find a default application for this file type.")]
    ApplicationMissing,
    #[error("Area too small, main window might not display correctly.")]
    SmallArea,
    #[error("Invalid input: {0}")]
    Input(String),
    #[error("File name prevents renaming with regex: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Error when directory walking: {0}")]
    IgnoreError(#[from] ignore::Error),
    #[error("Error when when watching files for changes: {0}")]
    NotifyError(#[from] notify::Error),
    #[error("Event without accompanying paths: {0:?}")]
    NotifyEventError(notify::Event),
    #[error("Failed to create parse options.")]
    ComrakError,
    #[error("Failed to parse YAML frontmatter.")]
    YamlError(#[from] yaml_rust::ScanError),
    #[error("Failed to find Git Repository.")]
    GitError(#[from] git2::Error),
}

impl RucolaError {
    pub fn to_ratatui(&self) -> Paragraph<'_> {
        Paragraph::new(format!("{}", &self)).style(style::Style::new().fg(style::Color::Red))
    }
}
