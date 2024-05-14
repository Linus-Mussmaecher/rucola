use std::path;

use clap::Parser;
use eyre::Context;

use crate::ui;

/// CLI arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// The target folder
    target_folder: Option<String>,
    /// Number of times to greet
    #[arg(short, long)]
    style: Option<String>,
}

/// Groups data passed by the user in the config file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConfigFile {
    /// Wether or not the select view filters while typing or only on enter.
    dynamic_filter: bool,
    /// Path to the vault to index.
    vault_path: Option<path::PathBuf>,
    /// Selected theme
    theme: String,
    /// The editor to use for notes
    editor: Option<String>,
    /// File endings to consider notes
    file_endings: Vec<String>,
    /// Default file ending for newly created notes
    default_ending: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            dynamic_filter: true,
            vault_path: None,
            theme: "default_light_theme".to_string(),
            editor: None,
            file_endings: vec![String::from("md")],
            default_ending: String::from("md"),
        }
    }
}

///A wrapper grouping all config files into one struct.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// The data stored in the config file
    config_file: ConfigFile,
    /// The data describing the look of the ui
    uistyles: ui::UiStyles,
}

impl Config {
    /// Loads a config file, looks for the specified theme and also loads it, and then groups both in a 'config' struct and returns that.
    pub fn load() -> color_eyre::Result<Self> {
        // === Step 1: Load config file ===
        let mut config_file: ConfigFile = confy::load("rucola", "config")
            .with_context(|| "Attempting to write/read config file.")?;

        // === Step 2: Read command line arguments
        let arguments = Arguments::parse();

        // Extract vault path. Expanduser expands `~` to the correct user home directory and similar.
        if let Some(arg_path_buf) = arguments
            .target_folder
            .and_then(|arg_string| expanduser::expanduser(arg_string).ok())
        {
            config_file.vault_path = Some(arg_path_buf);
        } else {
            config_file.vault_path = config_file.vault_path.and_then(|conf_path_buf| {
                expanduser::expanduser(conf_path_buf.to_string_lossy().to_string()).ok()
            })
        }

        // Check for a command line argument for the style
        config_file.theme = arguments.style.unwrap_or(config_file.theme);

        // === Step 3: Load style file ===
        let uistyles: ui::UiStyles = confy::load("rucola", config_file.theme.as_str())
            .with_context(|| "Attempting to write/read theme file.")?;

        Ok(Self {
            config_file,
            uistyles,
        })
    }

    /// Stores this config file in the default locations.
    /// As currently the config cannot be manipulated from within the program, this is unused.
    #[allow(dead_code)]
    pub fn store(self) -> color_eyre::Result<()> {
        confy::store("rucola", self.config_file.theme.as_str(), self.uistyles)?;
        confy::store("rucola", "config", self.config_file)?;

        Ok(())
    }

    /// Returns the user-selected UI styles.
    pub fn get_ui_styles(&self) -> &ui::UiStyles {
        &self.uistyles
    }

    /// Return the editor supposed to be used with notes.
    pub fn get_editor(&self) -> Option<&str> {
        self.config_file.editor.as_deref()
    }

    /// Return the valid file endings to be considered a note file.
    /// An empty string indicates that files with no extension ought to be accepted.
    pub fn get_endings(&self) -> &[String] {
        &self.config_file.file_endings
    }

    /// Returns the default file ending for a newly created note.
    pub fn get_default_extension(&self) -> &String {
        &self.config_file.default_ending
    }

    /// Returns the dynamic filtering option (wether to constantly refilter the selection list while the user types).
    pub fn get_dynamic_filter(&self) -> bool {
        self.config_file.dynamic_filter
    }

    /// Returns the default vault path.
    pub fn get_vault_path(&self) -> std::path::PathBuf {
        self.config_file
            .vault_path
            .clone()
            .unwrap_or(path::PathBuf::from("."))
    }
}
