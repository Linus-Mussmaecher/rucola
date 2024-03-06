use std::{fs::File, io::Read};

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
    pub fn from_file(mut file: File, name: String, path: String) -> color_eyre::Result<Self> {
        // Read content of markdown(plaintext) file
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Create regexes to match tags and links

        // Anything starting with a single #, then all non-whitespace chars until a whitespace
        let tag_regex = Regex::new(r"(\s|^)(\#[^\s\#]+)")?;
        // Anything between two sets of brackets. If the inner area is split by a |, only take the text before.
        let link_regex = Regex::new(r"\[\[([^\[\]\|]+)[\|]?[^\[\]\|]*\]\]")?;

        // Extract data
        Ok(Self {
            name,
            path,
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
