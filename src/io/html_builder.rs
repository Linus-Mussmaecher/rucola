use std::{fs, io::Write, path, process};

use crate::{data, error};

/// Struct that keeps configuration details for the creation of HTML files from markdown files.
#[derive(Debug, Clone)]
pub struct HtmlBuilder {
    /// Path to the vault to index.
    vault_path: path::PathBuf,
    /// When set to true, HTML files are mass-created on start and continuously kept up to date with file changes instead of being created on-demand.
    enable_html: bool,
    /// The resolved path to the css file, if there is one
    css_path: Option<path::PathBuf>,
    /// String to prepend to all generated html documents (e.g. for MathJax)
    html_prepend: Option<String>,
    /// Wether or not to insert a MathJax preamble in notes containing math code.
    mathjax: bool,
    /// A list of strings to replace in math mode to mimic latex commands
    math_replacements: Vec<(String, String)>,
    /// Viewer to open html files with
    viewer: Option<Vec<String>>,
}

impl Default for HtmlBuilder {
    fn default() -> Self {
        Self::new(
            &crate::Config::default(),
            std::env::current_dir().expect("Current directory to exist and be accessible."),
        )
    }
}

impl HtmlBuilder {
    pub fn new(config: &crate::Config, vault_path: path::PathBuf) -> Self {
        // Resolve css path
        let mut css_path = None;

        if let Some(css) = &config.css {
            if let Ok(mut css) = confy::get_configuration_file_path(
                "rucola",
                // remove css at the end, so no matter if the user included it or not, we always have the same format. If we left the css, confy would append .toml and we would end up with .css.css
                css.as_str().trim_end_matches(".css"),
            ) {
                // confy will append .toml (as this is the expected extension for config files)
                // so replace that with .css in any case.
                css.set_extension("css");
                css_path = Some(css);
            }
        }

        Self {
            vault_path,
            enable_html: config.enable_html,
            css_path,
            html_prepend: config.html_prepend.clone(),
            mathjax: config.mathjax,
            math_replacements: config.math_replacements.clone(),
            viewer: config.viewer.clone(),
        }
    }

    /// For a given note id, returns the path its HTML representation _would_ be stored at.
    /// Makes no guarantees if that representation currently exists.
    pub fn name_to_html_path(&self, name: &str) -> path::PathBuf {
        // calculate target path
        let mut tar_path = self.vault_path.clone();
        tar_path.push(".html/");

        tar_path.set_file_name(format!(".html/{}", &data::name_to_id(name)));
        tar_path.set_extension("html");
        tar_path
    }

    pub fn create_html(&self, note: &data::Note, force: bool) -> error::Result<()> {
        if !self.enable_html && !force {
            return Ok(());
        }

        // Read content of markdown(plaintext) file
        let content = fs::read_to_string(&note.path)?;

        // Parse markdown into AST
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            &content,
            &comrak::Options {
                extension: comrak::ExtensionOptionsBuilder::default()
                    .wikilinks_title_after_pipe(true)
                    .math_dollars(true)
                    .build()
                    .map_err(|_e| error::RucolaError::ComrakError)?,
                ..Default::default()
            },
        );

        let mut contains_math = false;

        for node in root.descendants() {
            // correct id urls for wiki links
            match node.data.borrow_mut().value {
                comrak::nodes::NodeValue::WikiLink(ref mut link) => {
                    link.url = format!("{}.html", data::name_to_id(&link.url));
                }
                comrak::nodes::NodeValue::Math(ref mut math) => {
                    contains_math = true;
                    let x = &mut math.literal;
                    // re-insert the dollar at beginning and end to make mathjax pick it up
                    x.insert(0, '$');
                    x.push('$');
                    // if display math, do it again.
                    if math.display_math {
                        x.insert(0, '$');
                        x.push('$');
                    }
                    *x = self.perform_replacements(x.clone());
                }
                _ => {}
            }
        }

        let tar_path = self.name_to_html_path(&note.name);

        // ensure parent exists
        if let Some(parent) = tar_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // get file (creates it if it doesn't exist)
        let mut tar_file = fs::File::create(&tar_path)?;

        writeln!(tar_file, "<title>{}</title>", note.name)?;
        self.add_preamble(&mut tar_file, contains_math)?;

        comrak::format_html(
            root,
            &comrak::Options {
                extension: comrak::ExtensionOptionsBuilder::default()
                    .wikilinks_title_after_pipe(true)
                    .math_dollars(true)
                    .build()
                    .map_err(|_e| error::RucolaError::ComrakError)?,
                ..Default::default()
            },
            &mut tar_file,
        )?;

        Ok(())
    }
    // Performs all string replacements as specified in the config file in the given string.
    pub fn perform_replacements(&self, mut initial_string: String) -> String {
        for (old, new) in self.math_replacements.iter() {
            initial_string = initial_string.replace(old, new);
        }
        initial_string
    }
    /// Prepends relevant data to a generated html file
    pub fn add_preamble(
        &self,
        html: &mut impl std::io::Write,
        contains_math: bool,
    ) -> error::Result<()> {
        // Prepend css location
        if let Some(css) = &self.css_path {
            writeln!(
                html,
                "<link rel=\"stylesheet\" href=\"{}\">",
                css.to_string_lossy()
            )?;
        }
        // Prepend mathjax code
        if contains_math && self.mathjax {
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
        if let Some(prep) = &self.html_prepend {
            html.write_all(prep.as_bytes())?;
        }
        Ok(())
    }
    /// Attempts to create a command to open the file at the given path to view it.
    /// Target should be an html file.
    /// Checks:
    ///  - The config file
    ///  - the systems default programms
    /// for an applicable program.
    pub fn create_view_command(&self, note: &data::Note) -> error::Result<std::process::Command> {
        let path = self.name_to_html_path(&note.name);
        // take the editor from the config file
        self.viewer
            .as_ref()
            // create a command from it
            .and_then(|viewer_arg_list| {
                let mut iter = viewer_arg_list.iter();
                if let Some(programm) = iter.next() {
                    let mut cmd = process::Command::new(programm);
                    for arg in iter {
                        if arg == "%p" {
                            // special argument for the user to indicate where to put the path
                            cmd.arg(&path);
                        } else {
                            // all other arguments are appended in order
                            cmd.arg(arg);
                        }
                    }
                    Some(cmd)
                } else {
                    None
                }
            })
            // if it was not there, take the default command
            .or_else(|| open::commands(&path).pop())
            // if it was also not there, throw an error
            .ok_or_else(|| error::RucolaError::ApplicationMissing)
    }
}

#[cfg(test)]
mod tests {

    use std::path::{Path, PathBuf};

    #[test]
    fn test_viewing() {
        let config = crate::Config::default();
        let fm = super::HtmlBuilder::new(&config, PathBuf::from("./tests"));
        let note =
            crate::data::Note::from_path(Path::new("./tests/common/notes/Books.md")).unwrap();

        fm.create_view_command(&note).unwrap();
    }

    #[test]
    fn test_create_html_no_panic() {
        let config = crate::Config::default();
        let hb = super::HtmlBuilder::new(&config, PathBuf::from("./tests"));

        let os =
            crate::data::Note::from_path(Path::new("./tests/common/notes/Operating Systems.md"))
                .unwrap();

        hb.create_html(&os, true).unwrap();
    }

    #[test]
    fn test_create_html_no_panic_math() {
        let config = crate::Config::default();
        let hb = super::HtmlBuilder::new(&config, PathBuf::from("./tests"));

        // with math
        let smooth_map =
            crate::data::Note::from_path(Path::new("./tests/common/notes/math/Smooth Map.md"))
                .unwrap();

        hb.create_html(&smooth_map, true).unwrap();
    }

    #[test]
    fn test_name_to_html_path() {
        let config = crate::Config::default();
        let hb = super::HtmlBuilder::new(&config, PathBuf::from("./tests"));

        assert_eq!(
            hb.name_to_html_path("Lie Group"),
            PathBuf::from("./tests/.html/lie-group.html")
        );
        assert_eq!(
            hb.name_to_html_path("lie-group"),
            PathBuf::from("./tests/.html/lie-group.html")
        );
        assert_eq!(
            hb.name_to_html_path("books"),
            PathBuf::from("./tests/.html/books.html")
        );
    }

    #[test]
    fn test_create_html_creates_files() {
        let config = crate::Config::default();
        let hb = super::HtmlBuilder::new(&config, PathBuf::from("./tests"));

        let books =
            crate::data::Note::from_path(Path::new("./tests/common/notes/Books.md")).unwrap();

        let b_path = hb.name_to_html_path("Books");

        if b_path.exists() {
            std::fs::remove_file(&b_path).unwrap();
        }

        // assert!(!b_path.exists());

        hb.create_html(&books, true).unwrap();

        assert!(b_path.exists());
    }

    #[test]
    fn test_create_html_creates_files_with_math() {
        let config = crate::Config::default();
        let hb = super::HtmlBuilder::new(&config, PathBuf::from("./tests"));

        // with math
        let liegroup =
            crate::data::Note::from_path(Path::new("./tests/common/notes/math/Lie Group.md"))
                .unwrap();

        let lg_path = hb.name_to_html_path("Lie Group");

        if Path::new(&lg_path).exists() {
            std::fs::remove_file(&lg_path).unwrap();
        }

        // assert!(!lg_path.exists());

        hb.create_html(&liegroup, true).unwrap();

        assert!(lg_path.exists());
    }

    #[test]
    fn test_replacements() {
        let config = crate::Config::default();
        let mut hb = super::HtmlBuilder::new(&config, PathBuf::from("./tests"));

        let field = "\\field{R} \neq \\field{C}".to_string();
        let topology = "\\topology{O} = \\topology{P}(X)".to_string();

        assert_eq!(
            hb.perform_replacements(field.clone()),
            "\\mathbb{R} \neq \\mathbb{C}"
        );
        assert_eq!(
            hb.perform_replacements(topology.clone()),
            "\\topology{O} = \\topology{P}(X)"
        );

        hb.math_replacements
            .push(("\\topology".to_string(), "\\mathcal".to_string()));

        assert_eq!(
            hb.perform_replacements(field.clone()),
            "\\mathbb{R} \neq \\mathbb{C}"
        );
        assert_eq!(
            hb.perform_replacements(topology.clone()),
            "\\mathcal{O} = \\mathcal{P}(X)"
        );
    }
}
