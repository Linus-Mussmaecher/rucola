/// Groups data passed by the user in the config file.
#[derive(Debug, Clone)]
pub struct Config {
    /// Wether or not the select view filters while typing or only on enter.
    pub dynamic_filter: bool,
    /// Path to the vault to index.
    pub vault_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dynamic_filter: true,
            vault_path: "/home/linus/Coppermind/".to_string(),
        }
    }
}
