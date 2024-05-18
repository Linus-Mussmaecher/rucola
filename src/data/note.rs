use std::{fs, io, path};

use itertools::Itertools;

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
    pub fn from_path(path: &path::Path) -> io::Result<Self> {
        // Open the file.
        let mut file = fs::File::open(path)?;

        // Read content of markdown(plaintext) file
        let mut content = String::new();
        io::Read::read_to_string(&mut file, &mut content)?;

        // Parse markdown into AST
        let arena = comrak::Arena::new();
        let root = comrak::parse_document(
            &arena,
            &content,
            &comrak::Options {
                extension: comrak::ExtensionOptionsBuilder::default()
                    .wikilinks_title_after_pipe(true)
                    .build()
                    .unwrap(),
                ..Default::default()
            },
        );

        Ok(Self {
            // Name: Remove file extension
            name: path
                .file_stem()
                .map(|os| os.to_string_lossy().to_string())
                .unwrap_or_default(),
            // Path: Already given - convert to owned version.
            path: path.to_path_buf(),
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
}
