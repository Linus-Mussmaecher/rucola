use ratatui::{prelude::*, widgets::*};
use std::{fmt::Debug, fs, path};

use itertools::Itertools;

use crate::{error, ui};

/// An abstract representation of a note that contains statistics about it but _not_ the full text.
#[derive(Clone, Debug, Default)]
pub struct Note {
    /// The title of the note.
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
}

impl Note {
    /// Opens the file from the given path (if possible) and extracts metadata.
    pub fn from_path(path: &path::Path) -> error::Result<Self> {
        // Open the file.
        let content = fs::read_to_string(path)?;

        // Create a regex to check for YAML front matter.
        let regex = regex::Regex::new("---\n((.|\n)*)\n---\n((.|\n)*)")?;

        // Extract both the YAML front matter, if present, and the main content.
        let (yaml, content) = if let Some(matches) = regex.captures(&content) {
            // If the regex matched, YAML front matter was present.
            (
                // The 1st capture group is the front matter.
                matches.get(1).map(|m| m.as_str().to_owned()),
                // The 3rd capture group is the actual content.
                matches.get(3).unwrap().as_str().to_owned(),
            )
        } else {
            // If the regex didn't match, then just use the content.
            (None, content)
        };

        // Parse markdown into AST
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            &content,
            &comrak::Options {
                extension: comrak::ExtensionOptions::builder()
                    .wikilinks_title_after_pipe(true)
                    .build(),
                ..Default::default()
            },
        );

        // Parse YAML.
        let title = if let Some(yaml) = yaml {
            let docs = yaml_rust::YamlLoader::load_from_str(&yaml)?;
            let doc = &docs[0];

            doc["title"].as_str().map(|s| s.to_owned())
        } else {
            None
        };

        Ok(Self {
            // Name: Check if there was one specified in the YAML fronmatter.
            // If not, remove file extension.
            name: title.unwrap_or(
                path.file_stem()
                    .map(|os| os.to_string_lossy().to_string())
                    .ok_or_else(|| error::RucolaError::NoteNameCannotBeRead(path.to_path_buf()))?,
            ),
            // Path: Already given - convert to owned version.
            path: path.canonicalize().unwrap_or(path.to_path_buf()),
            // Tags: Go though all text nodes in the AST, split them at whitespace and look for those starting with a hash.
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
        })
    }

    /// Converts this note to a small ratatui table displaying its most vital stats.
    pub fn to_stats_table(&self, styles: &ui::UiStyles) -> Table {
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
                Cell::from("Path:").style(styles.text_style),
                Cell::from(self.path.to_str().unwrap_or_default()).style(styles.text_style),
            ]),
        ];

        Table::new(stats_rows, stats_widths).column_spacing(1)
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
    fn test_yaml() {
        let note =
            crate::data::Note::from_path(Path::new("./tests/common/notes/note25.md")).unwrap();

        assert_eq!(note.name, String::from("YAML Format"));
        // assert_eq!(
        //     note.tags,
        //     vec![String::from("#diffgeo"), String::from("#topology")]
        // );
        assert_eq!(
            note.path,
            PathBuf::from("./tests/common/notes/note25.md")
                .canonicalize()
                .unwrap()
        );
    }
}
