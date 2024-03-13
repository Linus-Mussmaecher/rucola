use std::{fs::File, io::Read, path::Path};

use eyre::{Context, ContextCompat};
use regex::Regex;

#[derive(Clone, Debug)]
pub struct Note {
    pub name: String,
    pub path: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub words: usize,
    pub characters: usize,
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
            name: path
                .file_name()
                .map(|s| s.to_string_lossy().replace(".md", ""))
                .unwrap_or_else(|| "".to_string()),
            path: path.to_string_lossy().to_string(),
            tags: tag_regex
                .captures_iter(&content)
                .filter_map(|c| {
                    c.extract::<2>()
                        .1
                        .get(1)
                        .and_then(|ss| Some(String::from(*ss)))
                })
                .collect(),
            links: link_regex
                .captures_iter(&content)
                .filter_map(|c| {
                    c.extract::<1>()
                        .1
                        .first()
                        .and_then(|ss| Some(String::from(*ss).to_lowercase().replace(" ", "-")))
                })
                .collect(),
            words: content.split_whitespace().count(),
            characters: content.len(),
        })
    }
}
