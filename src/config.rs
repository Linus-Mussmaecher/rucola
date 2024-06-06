use std::path;

use crate::{error, ui};

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
    /// File types to consider notes
    /// See the [default list](https://docs.rs/ignore/latest/src/ignore/default_types.rs.html) of the ignore crate for possible options.
    /// The "all" option matches all files.
    file_types: Vec<String>,
    /// Default file ending for newly created notes
    default_extension: String,
    /// String to prepend to all generated html documents (e.g. for MathJax)
    html_prepend: Option<String>,
    /// Path to .css file to style htmls with.
    css: Option<String>,
    /// Viewer to open html files with
    viewer: Option<String>,
    /// Wether or not to insert a MathJax preamble in notes containing math code.
    mathjax: bool,
    /// A list of strings to replace in math mode to mimic latex commands
    math_replacements: Vec<(String, String)>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            dynamic_filter: true,
            mathjax: true,
            vault_path: if cfg!(test) {
                Some(path::PathBuf::from("./tests/common/notes/"))
            } else {
                None
            },
            theme: "default_light_theme".to_string(),
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

///A wrapper grouping all config files into one struct.
#[derive(Debug, Clone)]
pub struct Config {
    /// The data stored in the config file
    config_file: ConfigFile,
    /// The data describing the look of the ui
    uistyles: ui::UiStyles,
    /// The resolved path to the css file, if there is one
    css_path: Option<path::PathBuf>,
    /// Pre-calculated object containing allowed file types
    types: ignore::types::Types,
    /// Pre-fetched gitignore file in the vault path, if found.
    gitignore: Option<ignore::gitignore::Gitignore>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_file: ConfigFile::default(),
            uistyles: ui::UiStyles::default(),
            css_path: None,
            types: ignore::types::TypesBuilder::new()
                .add_defaults()
                .select("markdown")
                .build()
                .expect("Markdown is a valid file type configured in the DEFAULTs."),
            gitignore: None,
        }
    }
}

impl Config {
    /// Creates a finished config from valid ConfigFile and UiStyles structs.
    fn from_parts(
        config_file: ConfigFile,
        uistyles: ui::UiStyles,
    ) -> Result<Self, error::RucolaError> {
        // Resolve css path
        let mut css_path = None;

        if let Some(css) = &config_file.css {
            let mut css = confy::get_configuration_file_path(
                "rucola",
                // remove css at the end, so no matter if the user included it or not, we always have the same format. If we left the css, confy would append .toml and we would end up with .css.css
                css.as_str().trim_end_matches(".css"),
            )?;
            // confy will append .toml (as this is the expected extension for config files)
            // so replace that with .css in any case.
            css.set_extension("css");
            css_path = Some(css);
        }

        // Pre-calculate allowed file types
        let mut types_builder = ignore::types::TypesBuilder::new();
        types_builder.add_defaults();
        for name in config_file.file_types.iter() {
            types_builder.select(name);
        }

        // Search and fetch gitignore
        let gitignore_builder = ignore::gitignore::GitignoreBuilder::new(
            config_file
                .vault_path
                .as_ref()
                .unwrap_or(&path::PathBuf::from(".")),
        );

        Ok(Self {
            config_file,
            uistyles,
            css_path,
            types: types_builder.build()?,
            gitignore: gitignore_builder.build().ok(),
        })
    }

    /// Loads a config file, looks for the specified theme and also loads it, and then groups both in a 'config' struct and returns that.
    pub fn load(args: crate::Arguments) -> Result<Self, error::RucolaError> {
        // === Step 1: Load config file ===
        let mut config_file: ConfigFile = confy::load("rucola", "config")?;

        // === Step 2: Fix home path ===
        // Extract vault path. Expanduser expands `~` to the correct user home directory and similar.
        config_file.vault_path = args
            .target_folder
            // first attempt to extend the command line given path if one was passed
            .and_then(|arg_string| expanduser::expanduser(arg_string).ok())
            // if none was given, expand the path given from the config file
            .or_else(|| {
                config_file.vault_path.and_then(|conf_path_buf| {
                    expanduser::expanduser(conf_path_buf.to_string_lossy()).ok()
                })
            });

        // === Step 3: Load style file ===
        config_file.theme = args.style.unwrap_or(config_file.theme);

        let uistyles: ui::UiStyles = confy::load("rucola", config_file.theme.as_str())?;

        // Hand over to from_parts constructor to pre-calculate cached fields from loaded data
        Self::from_parts(config_file, uistyles)
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

    /// Attempts to create a command to open the file at the given path to edit it.
    /// Target should be a markdown file.
    /// Checks:
    ///  - The config file
    ///  - The $EDITOR environment variable
    ///  - the systems default programms
    /// for an applicable program.
    pub fn create_edit_command(
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
            .ok_or_else(|| error::RucolaError::ApplicationMissing)
    }

    /// Attempts to create a command to open the file at the given path to view it.
    /// Target should be an html file.
    /// Checks:
    ///  - The config file
    ///  - the systems default programms
    /// for an applicable program.
    pub fn create_view_command(
        &self,
        path: &path::PathBuf,
    ) -> Result<std::process::Command, error::RucolaError> {
        self.config_file
            // take the editor from the config file
            .viewer
            .as_ref()
            // create a command from it
            .map(|viewer_string| open::with_command(path, viewer_string))
            // if it was not there, take the default command
            .or_else(|| open::commands(path).pop())
            // if it was also not there, throw an error
            .ok_or_else(|| error::RucolaError::ApplicationMissing)
    }

    /// Returns a file walker that iterates over all notes to index.
    pub fn get_walker(&self) -> ignore::Walk {
        ignore::WalkBuilder::new(
            self.config_file
                .vault_path
                .as_ref()
                .unwrap_or(&path::PathBuf::from(".")),
        )
        .types(self.types.clone())
        .build()
    }

    /// Wether the given path is supposed to be tracked by rucola or not.
    /// Checks for file endings and (TODO) gitignore
    pub fn is_tracked(&self, path: &path::PathBuf) -> bool {
        let file_ending = if let ignore::Match::Whitelist(_) = self.types.matched(path, false) {
            true
        } else {
            false
        };

        let gitignore = self
            .gitignore
            .as_ref()
            .map(|gi| {
                if let ignore::Match::Ignore(_) = gi.matched(path, false) {
                    false
                } else {
                    true
                }
            })
            .unwrap_or(true);

        return file_ending && gitignore;
    }

    /// Takes in a PathBuf and, if the current file extension is not set, append the default one.
    pub fn ensure_file_extension(&self, path: &mut path::PathBuf) {
        if path.extension().is_none() {
            path.set_extension(&self.config_file.default_extension);
        }
    }

    /// Prepends relevant data to a generated html file
    pub fn add_preamble(
        &self,
        html: &mut impl std::io::Write,
        contains_math: bool,
    ) -> Result<(), error::RucolaError> {
        // Prepend css location
        if let Some(css) = &self.css_path {
            writeln!(
                html,
                "<link rel=\"stylesheet\" href=\"{}\">",
                css.to_string_lossy()
            )?;
        }
        // Prepend mathjax code
        if contains_math && self.config_file.mathjax {
            writeln!(
                html,
                r#"<script type="text/x-mathjax-config">MathJax.Hub.Config({{tex2jax: {{inlineMath: [ ['$','$'] ],processEscapes: true}}}});</script>"#
            )?;
            writeln!(
                html,
                r#"<script type="text/javascript"src="https://cdn.mathjax.org/mathjax/latest/MathJax.js?config=TeX-AMS-MML_HTMLorMML"></script>"#
            )?;
        }
        // Prepend all other manual configured prefixes
        if let Some(prep) = &self.config_file.html_prepend {
            html.write_all(prep.as_bytes())?;
        }
        Ok(())
    }

    // Performs all string replacements as specified in the config file in the given string.
    pub fn perform_replacements(&self, initial_string: &String) -> String {
        let mut res = initial_string.clone();
        for (old, new) in self.config_file.math_replacements.iter() {
            res = res.replace(old, new);
        }
        res
    }

    /// Returns the dynamic filtering option (wether to constantly refilter the selection list while the user types).
    pub fn get_dynamic_filter(&self) -> bool {
        self.config_file.dynamic_filter
    }

    /// Returns the default vault path.
    pub fn create_vault_path(&self) -> std::path::PathBuf {
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
            config.create_edit_command(&path.to_path_buf()).unwrap();
        }

        let config = Config {
            config_file: super::ConfigFile {
                editor: Some("helix".to_owned()),
                ..Default::default()
            },
            ..Default::default()
        };
        // if we use  a config with set editor path, we should also be able to create a command
        config.create_edit_command(&path.to_path_buf()).unwrap();
    }

    #[test]
    fn test_replacements() {
        let mut config = Config::default();

        let field = "\\field{R} \neq \\field{C}".to_string();
        let topology = "\\topology{O} = \\topology{P}(X)".to_string();

        assert_eq!(
            config.perform_replacements(&field),
            "\\mathbb{R} \neq \\mathbb{C}"
        );
        assert_eq!(
            config.perform_replacements(&topology),
            "\\topology{O} = \\topology{P}(X)"
        );

        config
            .config_file
            .math_replacements
            .push(("\\topology".to_string(), "\\mathcal".to_string()));

        assert_eq!(
            config.perform_replacements(&field),
            "\\mathbb{R} \neq \\mathbb{C}"
        );
        assert_eq!(
            config.perform_replacements(&topology),
            "\\mathcal{O} = \\mathcal{P}(X)"
        );
    }

    #[test]
    fn test_file_endings() {
        let no_ending_tar = std::path::PathBuf::from("./tests/common/test");
        let md_ending_tar = std::path::PathBuf::from("./tests/common/test.md");
        let txt_ending_tar = std::path::PathBuf::from("./tests/common/test.txt");
        let tex_ending_tar = std::path::PathBuf::from("./tests/common/test.tex");

        let config = Config::default();

        let mut no_ending = std::path::PathBuf::from("./tests/common/test");
        let mut md_ending = std::path::PathBuf::from("./tests/common/test.md");
        let mut txt_ending = std::path::PathBuf::from("./tests/common/test.txt");

        config.ensure_file_extension(&mut no_ending);
        config.ensure_file_extension(&mut md_ending);
        config.ensure_file_extension(&mut txt_ending);

        assert_eq!(no_ending, md_ending_tar);
        assert_eq!(md_ending, md_ending_tar);
        assert_eq!(txt_ending, txt_ending_tar);

        assert!(!config.is_tracked(&no_ending_tar));
        assert!(config.is_tracked(&md_ending_tar));
        assert!(!config.is_tracked(&txt_ending_tar));
        assert!(!config.is_tracked(&tex_ending_tar));

        let config = Config::from_parts(
            crate::config::ConfigFile {
                file_types: vec!["md".to_owned(), "txt".to_owned()],
                ..Default::default()
            },
            super::ui::UiStyles::default(),
        )
        .unwrap();

        assert!(!config.is_tracked(&no_ending_tar));
        assert!(config.is_tracked(&md_ending_tar));
        assert!(config.is_tracked(&txt_ending_tar));
        assert!(!config.is_tracked(&tex_ending_tar));

        let config = Config::from_parts(
            crate::config::ConfigFile {
                file_types: vec!["all".to_owned()],
                ..Default::default()
            },
            super::ui::UiStyles::default(),
        )
        .unwrap();

        assert!(!config.is_tracked(&no_ending_tar));
        assert!(config.is_tracked(&md_ending_tar));
        assert!(config.is_tracked(&txt_ending_tar));
        assert!(config.is_tracked(&tex_ending_tar));
    }
}
