use std::path;

use crate::{files, ui};

/// Describes when to show a which stats area.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StatsShow {
    // Always shows both stats
    Both,
    // Shows local stats when filtering and nothing otherwise
    Relevant,
    // Always shows only local stats
    Local,
}

/// Groups data passed by the user in the config file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Path to the vault to index.
    pub(crate) vault_path: Option<path::PathBuf>,
    /// File types to consider notes
    /// See the [default list](https://docs.rs/ignore/latest/src/ignore/default_types.rs.html) of the ignore crate for possible options.
    /// The "all" option matches all files.
    pub(crate) file_types: Vec<String>,
    /// Default file ending for newly created notes
    pub(crate) default_extension: String,
    /// Selected theme
    pub(crate) theme: String,
    /// When to show the global stats area
    pub(crate) stats_show: StatsShow,
    /// The editor to use for notes
    pub(crate) editor: Option<String>,
    /// Viewer to open html files with
    pub(crate) viewer: Option<String>,
    /// When set to true, HTML files are mass-created on start and continuously kept up to date with file changes instead of being created on-demand.
    pub(crate) enable_html: bool,
    /// Path to .css file to style htmls with.
    pub(crate) css: Option<String>,
    /// String to prepend to all generated html documents (e.g. for MathJax)
    pub(crate) html_prepend: Option<String>,
    /// Wether or not to insert a MathJax preamble in notes containing math code.
    pub(crate) mathjax: bool,
    /// A list of strings to replace in math mode to mimic latex commands
    pub(crate) math_replacements: Vec<(String, String)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_html: true,
            mathjax: true,
            vault_path: if cfg!(test) {
                Some(path::PathBuf::from("./tests/common/notes/"))
            } else {
                None
            },
            theme: "default_light_theme".to_string(),
            stats_show: StatsShow::Both,
            editor: None,
            file_types: vec![String::from("markdown")],
            default_extension: String::from("md"),
            html_prepend: None,
            css: None,
            viewer: None,
            math_replacements: vec![
                ("\\field".to_string(), "\\mathbb".to_string()),
                ("\\liealg".to_string(), "\\mathfrak".to_string()),
            ],
        }
    }
}

pub fn load_configurations(
    args: crate::Arguments,
) -> Result<
    (
        ui::UiStyles,
        files::HtmlBuilder,
        files::FileManager,
        files::FileTracker,
        StatsShow,
    ),
    crate::error::RucolaError,
> {
    // === Step 1: Load config file ===
    let mut config: Config = confy::load("rucola", "config")?;

    // === Step 2: Fix home path ===
    // Extract vault path. Expanduser expands `~` to the correct user home directory and similar.
    config.vault_path = args
        .target_folder
        // first attempt to extend the command line given path if one was passed
        .and_then(|arg_string| expanduser::expanduser(arg_string).ok())
        // if none was given, expand the path given from the config file
        .or_else(|| {
            config.vault_path.and_then(|conf_path_buf| {
                expanduser::expanduser(conf_path_buf.to_string_lossy()).ok()
            })
        })
        // make sure path is absolute
        .map(|path| {
            if !path.is_absolute() {
                std::env::current_dir().unwrap().join(path)
            } else {
                path
            }
        });

    // === Step 3: Load style file ===
    config.theme = args.style.unwrap_or(config.theme);

    let uistyles: ui::UiStyles = confy::load("rucola", config.theme.as_str())?;

    Ok((
        uistyles,
        files::HtmlBuilder::new(&config),
        files::FileManager::new(&config),
        files::FileTracker::new(&config),
        config.stats_show,
    ))
}
