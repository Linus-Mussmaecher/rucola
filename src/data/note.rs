use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use regex::Regex;

/// An abstract representation of a note that contains statistics about it but _not_ the full text.
#[derive(Clone, Debug)]
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
    pub path: PathBuf,
}

impl Note {
    /// Opens the file from the given path (if possible) and extracts metadata.
    pub fn from_path(path: &Path) -> io::Result<Self> {
        // Open the file.
        let mut file = File::open(path)?;

        // Read content of markdown(plaintext) file
        let mut content = String::new();
        io::Read::read_to_string(&mut file, &mut content)?;

        // Create regexes to match tags and links

        // Both regexes are hard-coded in, so we can expect as we know they are valid.
        // Anything starting with a single #, then all non-whitespace chars until a whitespace
        let tag_regex = Regex::new(r"(\s|^)(\#[^\s\#]+)").expect("Invalid static regex.");
        // Anything between two sets of brackets. If the inner area is split by a |, only take the text before.
        let link_regex = Regex::new(r"\[\[([^\[\]\|\#]+)[\#]?[\|]?[^\[\]\|]*\]\]")
            .expect("Invalid static regex.");

        Ok(Self {
            // Name: Remove file extension
            name: path
                .file_name()
                .map(|s| s.to_string_lossy().replace(".md", ""))
                .unwrap_or_else(|| "".to_string()),
            // Path: Already given - convert to owned version.
            path: path.to_path_buf(),
            // Tags: Use the regex to extract a list of captures, then convert them to strings
            tags: tag_regex
                .captures_iter(&content)
                .filter_map(|c| {
                    c.extract::<2>()
                        .1
                        .get(1)
                        .and_then(|ss| Some(String::from(*ss)))
                })
                .collect(),
            // Links: Use the regex to extract a list of captures, then convert them to strings
            links: link_regex
                .captures_iter(&content)
                .filter_map(|c| {
                    c.extract::<1>()
                        .1
                        .first()
                        .and_then(|ss| Some(String::from(*ss).to_lowercase().replace(" ", "-")))
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
