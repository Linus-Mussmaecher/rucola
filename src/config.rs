use std::path;

use clap::Parser;

use crate::{error, ui};

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
    file_extensions: Vec<String>,
    /// Default file ending for newly created notes
    default_extension: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            dynamic_filter: true,
            vault_path: None,
            theme: "default_light_theme".to_string(),
            editor: None,
            file_extensions: vec![String::from("md")],
            default_extension: String::from("md"),
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
    pub fn load() -> Result<Self, error::RucolaError> {
        // === Step 1: Load config file ===
        let mut config_file: ConfigFile = confy::load("rucola", "config")?;

        // === Step 2: Read command line arguments
        let arguments = Arguments::parse();

        // Extract vault path. Expanduser expands `~` to the correct user home directory and similar.
        config_file.vault_path = arguments
            .target_folder
            // first attempt to extend the command line given path if one was passed
            .and_then(|arg_string| expanduser::expanduser(arg_string).ok())
            // if none was given, expand the path given from the config file
            .or_else(|| {
                config_file.vault_path.and_then(|conf_path_buf| {
                    expanduser::expanduser(conf_path_buf.to_string_lossy().to_string()).ok()
                })
            });

        // Check for a command line argument for the style
        config_file.theme = arguments.style.unwrap_or(config_file.theme);

        // === Step 3: Load style file ===
        let uistyles: ui::UiStyles = confy::load("rucola", config_file.theme.as_str())?;

        Ok(Self {
            config_file,
            uistyles,
        })
    }

    /// Stores this config file in the default locations.
    /// As currently the config cannot be manipulated from within the program, this is unused.
    #[allow(dead_code)]
    pub fn store(self) -> Result<(), error::RucolaError> {
        confy::store("rucola", self.config_file.theme.as_str(), self.uistyles)?;
        confy::store("rucola", "config", self.config_file)?;

        Ok(())
    }

    /// Returns the user-selected UI styles.
    pub fn get_ui_styles(&self) -> &ui::UiStyles {
        &self.uistyles
    }

    /// Reads the config file and the
    pub fn create_opening_command(
        &self,
        path: &path::PathBuf,
    ) -> Result<std::process::Command, error::RucolaError> {
        self.config_file
            // take the editor from the config file
            .editor
            .as_ref()
            // create a command from it
            .map(|editor_string| open::with_command(path, editor_string))
            // Try the $EDITOR variable
            .or_else(|| {
                std::env::var("EDITOR")
                    .ok()
                    .map(|editor| open::with_command(path, editor))
            })
            // if it was not there, take the default command
            .or_else(|| open::commands(path).pop())
            // if it was also not there, throw an error
            .ok_or_else(|| error::RucolaError::EditorMissing)
    }

    /// Wether or not the given string constitutes a valid extension to be crawled by rucola.
    pub fn is_valid_extension(&self, ext: &str) -> bool {
        self.config_file
            .file_extensions
            .contains(&String::from(ext))
    }

    /// Takes in a PathBuf and, if the current file extension is not set, append the default one.
    pub fn validate_file_extension(&self, path: &mut path::PathBuf) {
        if path.extension().is_none() && !self.is_valid_extension("") {
            path.set_extension(&self.config_file.default_extension);
        }
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

#[cfg(test)]
mod tests {
    // use super::*;

    use crate::config::Config;

    #[test]
    fn test_opening() {
        let editor = std::env::var("EDITOR");

        let config = Config::default();
        let path = std::path::Path::new("./tests/common/notes/Books.md");

        if let Ok(_editor) = editor {
            // if we can unwrap the env variable, then we should be able to create a command
            config.create_opening_command(&path.to_path_buf()).unwrap();
        }

        let config = Config {
            config_file: super::ConfigFile {
                editor: Some("helix".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        };
        // if we use  a config with set editor path, we should also be able to create a command
        config.create_opening_command(&path.to_path_buf()).unwrap();
    }

    #[test]
    fn test_file_endings() {
        let no_ending_tar = std::path::PathBuf::from("./tests/common/test");
        let md_ending_tar = std::path::PathBuf::from("./tests/common/test.md");
        let txt_ending_tar = std::path::PathBuf::from("./tests/common/test.txt");

        let mut config = Config::default();

        let mut no_ending = std::path::PathBuf::from("./tests/common/test");
        let mut md_ending = std::path::PathBuf::from("./tests/common/test.md");
        let mut txt_ending = std::path::PathBuf::from("./tests/common/test.txt");

        config.validate_file_extension(&mut no_ending);
        config.validate_file_extension(&mut md_ending);
        config.validate_file_extension(&mut txt_ending);

        assert_eq!(no_ending, md_ending_tar);
        assert_eq!(md_ending, md_ending_tar);
        assert_eq!(txt_ending, txt_ending_tar);

        assert!(!config.is_valid_extension("txt"));
        assert!(!config.is_valid_extension(""));
        assert!(config.is_valid_extension("md"));

        config.config_file.file_extensions = vec!["md".to_owned(), "".to_owned()];

        let mut no_ending = std::path::PathBuf::from("./tests/common/test");
        let mut md_ending = std::path::PathBuf::from("./tests/common/test.md");
        let mut txt_ending = std::path::PathBuf::from("./tests/common/test.txt");

        config.validate_file_extension(&mut no_ending);
        config.validate_file_extension(&mut md_ending);
        config.validate_file_extension(&mut txt_ending);

        assert_eq!(no_ending, no_ending_tar);
        assert_eq!(md_ending, md_ending_tar);
        assert_eq!(txt_ending, txt_ending_tar);

        config.config_file.file_extensions = vec!["md".to_owned(), "*".to_owned()];

        let mut no_ending = std::path::PathBuf::from("./tests/common/test");
        let mut md_ending = std::path::PathBuf::from("./tests/common/test.md");
        let mut txt_ending = std::path::PathBuf::from("./tests/common/test.txt");

        config.validate_file_extension(&mut no_ending);
        config.validate_file_extension(&mut md_ending);
        config.validate_file_extension(&mut txt_ending);

        assert_eq!(no_ending, md_ending);
        assert_eq!(md_ending, md_ending_tar);
        assert_eq!(txt_ending, txt_ending_tar);
    }
}
