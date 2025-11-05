use std::{collections::HashMap, path};

use crate::{data, error, ui};

/// The file format a viewer expects.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum ViewerType {
    /// A viewer that displays HTML files and thus needs to be given paths to an HTML file.
    #[default]
    Html,
    /// A viewer that displays markdown files and thus needs to be given paths to an markdown file.
    Markdown,
}

/// Groups data passed by the user in the config file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    /// Path to the vault to index.
    pub(crate) vault_path: Option<path::PathBuf>,
    /// File types to consider notes
    /// See the [default list](https://docs.rs/ignore/latest/src/ignore/default_types.rs.html) of the ignore crate for possible options.
    /// The "all" option matches all files.
    pub(crate) file_types: Vec<String>,
    /// Default file ending for newly created notes
    pub(crate) default_extension: String,
    /// Whether to create a cache file of the index to reload on program start.
    pub(crate) cache_index: bool,
    /// Selected theme
    pub(crate) theme: String,
    /// When to show the global stats area
    pub(crate) stats_show: ui::screen::StatsShow,
    /// Whether tags have to match exactly or only by prefix when filtering.
    pub(crate) tag_match: data::TagMatch,
    /// Default sorting mode for notes
    pub(crate) default_sorting: data::SortingMode,
    /// Default sorting direction (true for ascending, false for descending)
    pub(crate) default_sorting_asc: bool,
    /// The editor to use for notes.
    pub(crate) editor: Option<Vec<String>>,
    /// Main viewer to inspect rendered notes.
    pub(crate) viewer: Option<Vec<String>>,
    /// Preferred file type of the main viewer.
    pub(crate) viewer_type: Option<ViewerType>,
    /// Alternative viewer to inspect rendered notes.
    pub(crate) secondary_viewer: Option<Vec<String>>,
    /// Preferred file type of the alternative viewer.
    pub(crate) secondary_viewer_type: Option<ViewerType>,
    /// When set to true, HTML files are mass-created on start and continuously kept up to date with file changes instead of being created on-demand.
    pub(crate) enable_html: bool,
    /// Path to .css file to style htmls with.
    pub(crate) css: Option<String>,
    /// String to prepend to all generated html documents (e.g. for MathJax)
    pub(crate) html_prepend: Option<String>,
    /// Whether or not to insert a MathJax preamble in notes containing math code.
    pub(crate) katex: bool,
    /// A list of strings to replace in math mode to mimic latex commands
    pub(crate) math_replacements: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault_path: None,
            file_types: vec![String::from("markdown")],
            default_extension: String::from("md"),
            cache_index: false,
            theme: "default_dark".to_string(),
            stats_show: ui::screen::StatsShow::Both,
            tag_match: data::TagMatch::Exact,
            default_sorting: data::SortingMode::Name,
            default_sorting_asc: true,
            editor: None,
            viewer_type: Some(ViewerType::Html),
            viewer: Some(vec![String::from("firefox"), String::from("%p")]),
            secondary_viewer_type: None,
            secondary_viewer: None,
            enable_html: true,
            css: Some("default_dark".to_string()),
            html_prepend: None,
            katex: true,
            math_replacements: HashMap::from_iter(vec![(
                "\\field".to_string(),
                "\\mathbb".to_string(),
            )]),
        }
    }
}

impl Config {
    /// Creates a config file and vault path by combining the passed cli arguments with the loaded file from comfy.
    pub fn load(args: crate::Arguments) -> error::Result<Self> {
        // === Step 1: Load config file ===
        let mut config: Config = confy::load("rucola", "config")?;

        // === Step 2: Fix vault path ===

        // Get current dir & extract vault path.
        let mut full_vault_path = Self::vault_path(std::env::current_dir()?, args, &mut config);

        // make sure path is absolute
        if !full_vault_path.is_absolute() {
            full_vault_path = std::env::current_dir()?.join(full_vault_path);
        }

        config.vault_path = Some(full_vault_path);

        Ok(config)
    }

    /// Not expansion on windows
    #[cfg(not(target_family = "unix"))]
    fn vault_path(
        pwd: path::PathBuf,
        args: crate::Arguments,
        config: &mut Config,
    ) -> path::PathBuf {
        args.target_folder
            .map(|folder_string| path::PathBuf::from(folder_string))
            .or(config.vault_path.take())
            .unwrap_or_else(|| pwd.clone())
    }

    /// Expanduser expands `~` to the correct user home directory and similar, on unix systems.
    #[cfg(target_family = "unix")]
    fn vault_path(
        pwd: path::PathBuf,
        args: crate::Arguments,
        config: &mut Config,
    ) -> path::PathBuf {
        args.target_folder
            // first attempt to extend the command line given path if one was passed
            .and_then(|arg_string| expanduser::expanduser(arg_string).ok())
            // if none was given, expand the path given from the config file
            .or_else(|| {
                config.vault_path.take().and_then(|conf_path_buf| {
                    expanduser::expanduser(conf_path_buf.to_string_lossy()).ok()
                })
            })
            .unwrap_or_else(|| pwd.clone())
    }
}
