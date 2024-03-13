use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use eyre::Context;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct Note {
    pub name: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub words: usize,
    pub characters: usize,
    pub path: PathBuf,
}

impl Note {
    /// Opens the file from the given path (if possible) and extracts metadata.
    pub fn from_path(path: &Path) -> color_eyre::Result<Self> {
        // Open the file.
        let mut file = File::open(path).with_context(|| "When trying to load a note file.")?;

        // Read content of markdown(plaintext) file
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Create regexes to match tags and links

        // Anything starting with a single #, then all non-whitespace chars until a whitespace
        let tag_regex = Regex::new(r"(\s|^)(\#[^\s\#]+)")?;
        // Anything between two sets of brackets. If the inner area is split by a |, only take the text before.
        let link_regex = Regex::new(r"\[\[([^\[\]\|]+)[\|]?[^\[\]\|]*\]\]")?;

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
