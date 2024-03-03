mod note;

use eyre::Context;
use std::{collections::HashMap, fs::File, path::Path};
use walkdir;

pub use note::Note;

/// Reads a passed directory recursively, returning a hashmap containing
///  - An entry for every '.md' file in the directory or any subdirectories
///  - The key will be the file name, without the file extension, in lowercase and with spaces replaced by dashes
///  - The value will be an instance of Note containing metadata of the file.
pub fn create_index(directory: &Path) -> color_eyre::Result<HashMap<String, Note>> {
    let mut index = HashMap::new();

    for e in walkdir::WalkDir::new(directory)
        .into_iter()
        // Ignore dot-folders and dotfiles
        .filter_entry(is_not_hidden)
        // Check only OKs
        .flatten()
        .filter(is_markdown)
    {
        // Attempt to load the file

        if let Ok(file) = File::open(e.path()) {
            index.insert(
                // Convert file name to lowercase dash separated version
                e.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .replace(" ", "-")
                    .replace(".md", ""),
                // Extract other metadata. Pass down original file name.
                Note::from_file(
                    file,
                    e.file_name().to_string_lossy().replace(".md", ""),
                    e.path()
                        .to_string_lossy()
                        .strip_prefix(&directory.to_string_lossy().as_ref())
                        .unwrap_or("No path")
                        .to_owned(),
                )
                .wrap_err_with(|| "Attempting to index all markdown files.")?,
            );
        }
    }

    Ok(index)
}

/// Checks if the given dir entry is 'hidden', i.e. not the root of a search and prefixed by a dot.
fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}
/// Checks if the given dir entry is a markdown file, i.e. not a directory and ends in '.md'
fn is_markdown(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_file() && entry.file_name().to_string_lossy().ends_with(".md")
}
