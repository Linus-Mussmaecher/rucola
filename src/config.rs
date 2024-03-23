use crate::ui;

/// Groups data passed by the user in the config file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConfigFile {
    /// Wether or not the select view filters while typing or only on enter.
    dynamic_filter: bool,
    /// Path to the vault to index.
    vault_path: String,
    /// Selected theme
    theme: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            dynamic_filter: true,
            vault_path: "/home/linus/Coppermind/".to_string(),
            theme: "default_light".to_string(),
        }
    }
}

///A wrapper grouping all config files into one struct
#[derive(Debug, Clone)]
pub struct Config {
    /// The data stored in the config file
    config_file: ConfigFile,
    /// The data describing the look of the ui
    styles: ui::Styles,
}

impl Config {
    /// Loads a config file, looks for the specified theme and also loads it, and then groups both in a 'config' struct and returns that.
    pub fn load() -> color_eyre::Result<Self> {
        let config_file: ConfigFile = confy::load("giraffe", "config")?;

        let styles: ui::Styles = confy::load("giraffe", config_file.theme.as_str())?;

        Ok(Self {
            config_file,
            styles,
        })
    }

    /// Stores this config file in the default locations
    pub fn store(self) -> color_eyre::Result<()> {
        confy::store("giraffe", self.config_file.theme.as_str(), self.styles)?;
        confy::store("giraffe", "config", self.config_file)?;

        Ok(())
    }

    /// Returns the user-selected UI styles
    pub fn get_styles(&self) -> &ui::Styles {
        &self.styles
    }

    /// Returns the dynamic filtering option
    pub fn get_dynamic_filter(&self) -> bool {
        self.config_file.dynamic_filter
    }

    /// Returns the default vault path
    pub fn get_vault_path(&self) -> &str {
        &self.config_file.vault_path
    }
}
