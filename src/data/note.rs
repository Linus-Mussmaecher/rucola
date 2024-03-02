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
    pub fn from_file(mut file: File, name: String, path: String) -> std::io::Result<Self> {
        //TODO: error handling
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let tag_regex = Regex::new(r"(\#[^\s\#]+)\s").unwrap();
        let link_regex = Regex::new(r"\[\[([^\[\]\|]+)[\|]?[^\[\]\|]*\]\]").unwrap();

        Ok(Self {
            name,
            path,
            tags: tag_regex
                .captures_iter(&content)
                .filter_map(|c| {
                    c.extract::<1>()
                        .1
                        .first()
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
