use ratatui::{prelude::*, widgets::*};
use std::{fmt::Debug, fs, path, time};

use itertools::Itertools;

use crate::{error, ui};

/// An abstract representation of a note that contains statistics about it but _not_ the full text.
#[derive(Clone, Debug, Default)]
pub struct Note {
    /// The title of the note.
    pub display_name: String,
    /// The name of the file the note is saved in.
    pub name: String,
    /// All tags contained at any part of the note.
    pub tags: Vec<String>,
    /// All links contained within the note - no external (e.g. web) links.
    pub links: Vec<String>,
    /// The number of words.
    pub words: usize,
    /// The number of characters.
    pub characters: usize,
    /// A copy of the path leading to this note.
    pub path: path::PathBuf,
    /// The date and time when the note was last modified.
    pub last_modification: Option<time::SystemTime>,
    /// Wether or not the note contains (valid) YAML frontmatter. If it does, this is the index of the beginning of the actual content.
    pub yaml_frontmatter: Option<usize>,
}

impl Note {
    /// Opens the file from the given path (if possible) and extracts metadata.
    pub fn from_path(path: &path::Path) -> error::Result<Self> {
        // Open the file.
        let content = fs::read_to_string(path)?;

        // Attempt to identify YAML frontmatter
        let (title, tags, begin_content) =
        // File needs to start with three dashes.
        if content.starts_with("---\n") {
            // Then search for the next three dashes.
            let break_position = content.find("\n---\n");
            // If they exist,
            if let Some(break_position) = break_position {
                // Take everything in between
                let possible_frontmatter = content.split_at(break_position).0;
                // Attempt to parse it as YAML.
                if let Ok((title, tags)) =
                    Self::parse_yaml(possible_frontmatter.trim_start_matches("---\n"))
                {
                    // If it worked, return the parsed data and the start of the actual note.
                    (title, tags, Some(break_position + 5))
                } else {
                    // Fail case: Parsing failed.
                    (None, Vec::new(), None)
                }
            } else {
                // Fail case: File has no second ---.
                (None, Vec::new(), None)
            }
        } else {
            // Fail case: File doesn't start with ---.
            (None, Vec::new(), None)
        };

        // Parse markdown into AST
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            content.split_at(begin_content.unwrap_or(0)).1,
            &comrak::Options {
                extension: comrak::ExtensionOptions::builder()
                    .wikilinks_title_after_pipe(true)
                    .build(),
                ..Default::default()
            },
        );

        // Parse YAML.

        Ok(Self {
            // Name: Check if there was one specified in the YAML fronmatter.
            // If not, remove file extension.
            display_name: title.unwrap_or(
                path.file_stem()
                    .map(|os| os.to_string_lossy().to_string())
                    .ok_or_else(|| error::RucolaError::NoteNameCannotBeRead(path.to_path_buf()))?,
            ),
            // File name: Remove file extension.
            name: path
                .file_stem()
                .map(|os| os.to_string_lossy().to_string())
                .ok_or_else(|| error::RucolaError::NoteNameCannotBeRead(path.to_path_buf()))?,
            // Path: Already given - convert to owned version.
            path: path.canonicalize().unwrap_or(path.to_path_buf()),
            // Modification: Can be read from the metadata of the path.
            last_modification: path.metadata().and_then(|m| m.modified()).ok(),
            // Tags: Go though all text nodes in the AST, split them at whitespace and look for those starting with a hash.
            // Finally, append tags specified in the YAML frontmatter.
            tags: root
                .descendants()
                .flat_map(|node| match &node.data.borrow().value {
                    comrak::nodes::NodeValue::Text(content) => content
                        .split_whitespace()
                        .filter(|s| s.starts_with('#'))
                        .map(|s| s.to_owned())
                        .collect_vec(),
                    _ => vec![],
                })
                .chain(tags)
                .collect(),
            // Links: Go though all wikilinks in the syntax tree and map them
            links: root
                .descendants()
                .flat_map(|node| match &node.data.borrow().value {
                    comrak::nodes::NodeValue::WikiLink(link) => Some(super::name_to_id(&link.url)),
                    comrak::nodes::NodeValue::Link(link) => {
                        if !link.url.contains('/') && !link.url.contains('.') {
                            Some(super::name_to_id(&link.url))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .collect(),
            // Words: Split at whitespace, grouping multiple consecutive instances of whitespace together.
            // See definition of `split_whitespace` for criteria.
            words: content.split_whitespace().count(),
            // Characters: Simply use the length of the string.
            characters: content.len(),
            // YAML: We already set this bool.
            yaml_frontmatter: begin_content,
        })
    }

    /// Converts this note to a small ratatui table displaying its most vital stats.
    pub fn to_stats_table(&self, styles: &ui::UiStyles) -> Table<'_> {
        let stats_widths = [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Min(20),
        ];

        // Display the note's tags
        let tags = self
            .tags
            .iter()
            .enumerate()
            .flat_map(|(index, s)| {
                [
                    Span::styled(if index == 0 { "" } else { ", " }, styles.text_style),
                    Span::styled(s.as_str(), styles.subtitle_style),
                ]
            })
            .collect_vec();

        // Stats Area
        let stats_rows = [
            Row::new(vec![
                Cell::from("Words:").style(styles.text_style),
                Cell::from(format!("{:7}", self.words)).style(styles.text_style),
                Cell::from("Tags:").style(styles.text_style),
                Cell::from(Line::from(tags)).style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("Chars:").style(styles.text_style),
                Cell::from(format!("{:7}", self.characters)).style(styles.text_style),
                Cell::from("Changed:").style(styles.text_style),
                Cell::from(
                    self.last_modification
                        .map(|st| {
                            Into::<chrono::DateTime<chrono::offset::Local>>::into(st)
                                .format("%Y-%m-%d %H:%M")
                                .to_string()
                        })
                        .unwrap_or("".to_owned().to_string()),
                )
                .style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("").style(styles.text_style),
                Cell::from("").style(styles.text_style),
                Cell::from("Path:").style(styles.text_style),
                Cell::from(self.path.to_str().unwrap_or_default()).style(styles.text_style),
            ]),
        ];

        Table::new(stats_rows, stats_widths).column_spacing(1)
    }

    /// Takes a str that possibly contains YAML frontmatter and attempts to parse it into a title and a list of tags.
    fn parse_yaml(yaml: &str) -> Result<(Option<String>, Vec<String>), yaml_rust::ScanError> {
        let docs = yaml_rust::YamlLoader::load_from_str(yaml)?;
        let doc = &docs[0];

        // Check if there was a title specified.
        let title = doc["title"].as_str().map(|s| s.to_owned());

        // Check if tags were specified.
        let tags = doc["tags"]
            // Convert the entry into a vec - if the entry isn't there, use an empty vec.
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            // Convert the individual entries into strs, as rust-yaml doesn't do nested lists.
            .flat_map(|v| v.as_str())
            // Convert those into Strings and prepend the #.
            .flat_map(|s| {
                // Entries of sublists will appear as separated by ` - `, so split by that.
                let parts = s.split(" - ").collect_vec();

                if parts.is_empty() {
                    // This should not happen.
                    Vec::new()
                } else if parts.len() == 1 {
                    // Only one parts => There were not subtags. Simply prepend a `#`.
                    vec![format!("#{}", s)]
                } else {
                    // More than 1 part => There were subtags.
                    let mut res = Vec::new();

                    // Iterate through all of the substrings except for the first, which is the supertag.
                    for subtag in parts.iter().skip(1) {
                        res.push(format!("#{}/{}", parts[0], subtag));
                    }

                    res
                }
            })
            // Collect all tags in a vec.
            .collect_vec();

        Ok((title, tags))
    }
}

#[cfg(test)]
mod tests {

    use std::path::{Path, PathBuf};

    #[test]
    fn test_loading() {
        let _note =
            crate::data::Note::from_path(Path::new("./tests/common/notes/Books.md")).unwrap();
    }

    #[test]
    fn test_values() {
        let note =
            crate::data::Note::from_path(Path::new("./tests/common/notes/math/Chart.md")).unwrap();

        assert_eq!(note.name, String::from("Chart"));
        assert_eq!(
            note.tags,
            vec![String::from("#diffgeo"), String::from("#topology")]
        );
        assert_eq!(
            note.links,
            vec![String::from("manifold"), String::from("diffeomorphism")]
        );
        assert_eq!(note.words, 115);
        assert_eq!(note.characters, 678);
        assert_eq!(
            note.path,
            PathBuf::from("./tests/common/notes/math/Chart.md")
                .canonicalize()
                .unwrap()
        );
    }

    #[test]
    fn test_yaml_name() {
        let note =
            crate::data::Note::from_path(Path::new("./tests/common/notes/note25.md")).unwrap();

        assert_eq!(note.display_name, String::from("YAML Format"));
        assert_eq!(note.name, String::from("note25"));

        assert_eq!(
            note.path,
            PathBuf::from("./tests/common/notes/note25.md")
                .canonicalize()
                .unwrap()
        );
    }

    #[test]
    fn test_yaml_tags() {
        let note =
            crate::data::Note::from_path(Path::new("./tests/common/notes/note25.md")).unwrap();

        assert_eq!(
            note.tags,
            vec![
                String::from("#test"),
                String::from("#files/yaml"),
                String::from("#files/markdown"),
                String::from("#abbreviations")
            ]
        );
    }
}
