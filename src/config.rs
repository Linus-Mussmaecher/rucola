use eyre::Context;

use crate::ui;

/// Groups data passed by the user in the config file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConfigFile {
    /// Wether or not the select view filters while typing or only on enter.
    dynamic_filter: bool,
    /// Path to the vault to index.
    vault_path: Option<String>,
    /// Selected theme
    theme: String,
    /// The editor to use for notes
    editor: Option<String>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            dynamic_filter: true,
            vault_path: None,
            theme: "default_light_theme".to_string(),
            editor: None,
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
        let config_file: ConfigFile = confy::load("rucola", "config")
            .with_context(|| "Attempting to write/read config file.")?;

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

    /// Returns the dynamic filtering option (wether to constantly refilter the selection list while the user types).
    pub fn get_dynamic_filter(&self) -> bool {
        self.config_file.dynamic_filter
    }

    /// Returns the default vault path.
    pub fn get_vault_path(&self) -> Option<&str> {
        self.config_file.vault_path.as_deref()
    }
}
