use std::{collections::HashMap, fs, io::Write, path};

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
    katex: bool,
    /// A list of strings to replace in math mode to mimic latex commands
    math_replacements: HashMap<String, String>,
}

impl Default for HtmlBuilder {
    fn default() -> Self {
        Self::new(&crate::Config::default())
    }
}

impl HtmlBuilder {
    pub fn new(config: &crate::Config) -> Self {
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
                // ensure that a file exists
                if !css.exists() {
                    let _ = std::fs::File::create(&css);
                }
                // set the value
                css_path = Some(css);
            }
        }

        Self {
            vault_path: config
                .vault_path
                .clone()
                .expect("Vault path should be set."),
            enable_html: config.enable_html,
            css_path,
            html_prepend: config.html_prepend.clone(),
            katex: config.katex,
            math_replacements: config.math_replacements.clone(),
        }
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
            content.split_at(note.yaml_frontmatter.unwrap_or(0)).1,
            &comrak::Options {
                extension: comrak::ExtensionOptions::builder()
                    .wikilinks_title_after_pipe(true)
                    .math_dollars(true)
                    .build(),
                ..Default::default()
            },
        );

        let mut contains_math = false;
        let mut contains_code = false;

        for node in root.descendants() {
            // correct id urls for wiki links
            match node.data.borrow_mut().value {
                comrak::nodes::NodeValue::WikiLink(ref mut link) => {
                    link.url = format!("{}.html", data::name_to_id(&link.url));
                }
                comrak::nodes::NodeValue::Link(ref mut link) => {
                    if !link.url.contains('/') && !link.url.contains('.') {
                        link.url = format!("{}.html", data::name_to_id(&link.url));
                    }
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
                }
                comrak::nodes::NodeValue::CodeBlock(ref _code) => {
                    contains_code = true;
                }
                _ => {}
            }
        }

        let tar_path = name_to_html_path(&note.name, &self.vault_path);

        // ensure parent exists
        if let Some(parent) = tar_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // get file (creates it if it doesn't exist)
        let mut tar_file = fs::File::create(&tar_path)?;

        writeln!(tar_file, "<!DOCTYPE html>")?;
        writeln!(tar_file, "<title>{}</title>", note.name)?;
        self.add_preamble(&mut tar_file, contains_math, contains_code)?;

        // Create a buffered writer to speed up the file writing process
        let mut tar_file_buffer = std::io::BufWriter::new(&mut tar_file);

        comrak::format_html(
            root,
            &comrak::Options {
                extension: comrak::ExtensionOptions::builder()
                    .wikilinks_title_after_pipe(true)
                    .math_dollars(true)
                    .build(),
                ..Default::default()
            },
            &mut tar_file_buffer,
        )?;

        Ok(())
    }

    /// Prepends relevant data to a generated html file
    pub fn add_preamble(
        &self,
        html: &mut impl std::io::Write,
        contains_math: bool,
        contains_code: bool,
    ) -> error::Result<()> {
        // Prepend css location
        if let Some(css) = &self.css_path {
            writeln!(
                html,
                "<link rel=\"stylesheet\" href=\"file://{}\">",
                css.to_string_lossy()
            )?;
        }
        // Prepend mathjax code
        if contains_math && self.katex {
            writeln!(
                html,
                r#"<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.10/dist/katex.min.css" integrity="sha384-wcIxkf4k558AjM3Yz3BBFQUbk/zgIYC2R0QpeeYb+TwlBVMrlgLqwRjRtGZiK7ww" crossorigin="anonymous">"#
            )?;
            writeln!(
                html,
                r#"<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.10/dist/katex.min.js" integrity="sha384-hIoBPJpTUs74ddyc4bFZSM1TVlQDA60VBbJS0oA934VSz82sBx1X7kSx2ATBDIyd" crossorigin="anonymous"></script>"#
            )?;
            writeln!(
                html,
                r#"<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.10/dist/contrib/auto-render.min.js" integrity="sha384-43gviWU0YVjaDtb/GhzOouOXtZMP/7XUzwPTstBeZFe/+rCMvRwr4yROQP43s0Xk" crossorigin="anonymous"></script>"#
            )?;
            writeln!(
                html,
                r##"<script>
    document.addEventListener("DOMContentLoaded", function() {{
        renderMathInElement(document.body, {{
          delimiters: [
              {{left: '$$', right: '$$', display: true}},
              {{left: '$', right: '$', display: false}},
          ],
          macros: {},
          throwOnError : false
        }});
    }});
</script>"##,
                serde_json::to_string(&self.math_replacements).unwrap()
            )?;
        }

        if contains_code {
            writeln!(
                html,
                r##"<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/default.min.css">"##
            )?;
            writeln!(
                html,
                r##"<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>"##
            )?;
            writeln!(
                html,
                r##"<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/go.min.js"></script>"##
            )?;
            writeln!(html, r##"<script>hljs.highlightAll();</script>"##)?;
        }

        // Prepend all other manual configured prefixes
        if let Some(prep) = &self.html_prepend {
            html.write_all(prep.as_bytes())?;
        }
        Ok(())
    }
}

/// For a given note id, returns the path its HTML representation _would_ be stored at.
/// Makes no guarantees if that representation currently exists.
pub fn name_to_html_path(name: &str, vault_path: &path::Path) -> path::PathBuf {
    // calculate target path
    let mut tar_path = vault_path.to_path_buf();
    tar_path.push(".html/");

    tar_path.set_file_name(format!(".html/{}", &data::name_to_id(name)));
    tar_path.set_extension("html");
    tar_path
}

#[cfg(test)]
mod tests {

    use std::path::{Path, PathBuf};

    #[test]
    fn test_create_html_no_panic() {
        let mut config = crate::Config::default();
        config.vault_path = Some(std::path::PathBuf::from("./tests"));
        let hb = super::HtmlBuilder::new(&config);

        let os =
            crate::data::Note::from_path(Path::new("./tests/common/notes/Operating Systems.md"))
                .unwrap();

        hb.create_html(&os, true).unwrap();
    }

    #[test]
    fn test_create_html_no_panic_math() {
        let mut config = crate::Config::default();
        config.vault_path = Some(std::path::PathBuf::from("./tests"));
        let hb = super::HtmlBuilder::new(&config);

        // with math
        let smooth_map =
            crate::data::Note::from_path(Path::new("./tests/common/notes/math/Smooth Map.md"))
                .unwrap();

        hb.create_html(&smooth_map, true).unwrap();
    }

    #[test]
    fn test_name_to_html_path() {
        // let config = crate::Config::default();
        let vault_path = PathBuf::from("./tests");

        assert_eq!(
            super::name_to_html_path("Lie Group", &vault_path),
            PathBuf::from("./tests/.html/lie-group.html")
        );
        assert_eq!(
            super::name_to_html_path("lie-group", &vault_path),
            PathBuf::from("./tests/.html/lie-group.html")
        );
        assert_eq!(
            super::name_to_html_path("books", &vault_path),
            PathBuf::from("./tests/.html/books.html")
        );
    }

    #[test]
    fn test_create_html_creates_files() {
        let mut config = crate::Config::default();
        let vault_path = PathBuf::from("./tests");
        let b_path = super::name_to_html_path("Books", &vault_path);
        config.vault_path = Some(vault_path);
        let hb = super::HtmlBuilder::new(&config);

        let books =
            crate::data::Note::from_path(Path::new("./tests/common/notes/Books.md")).unwrap();

        if b_path.exists() {
            std::fs::remove_file(&b_path).unwrap();
        }

        // assert!(!b_path.exists());

        hb.create_html(&books, true).unwrap();

        assert!(b_path.exists());
    }

    #[test]
    fn test_create_html_creates_files_with_math() {
        let mut config = crate::Config::default();
        let vault_path = PathBuf::from("./tests");
        let lg_path = super::name_to_html_path("Lie Group", &vault_path);
        config.vault_path = Some(vault_path);
        let hb = super::HtmlBuilder::new(&config);

        // with math
        let liegroup =
            crate::data::Note::from_path(Path::new("./tests/common/notes/math/Lie Group.md"))
                .unwrap();

        if Path::new(&lg_path).exists() {
            std::fs::remove_file(&lg_path).unwrap();
        }

        // assert!(!lg_path.exists());

        hb.create_html(&liegroup, true).unwrap();

        assert!(lg_path.exists());
    }
}
