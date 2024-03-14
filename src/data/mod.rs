mod note;
pub use note::Note;

mod note_statistics;
pub use note_statistics::EnvironmentStats;
pub use note_statistics::Filter;

use std::{collections::HashMap, path::Path};
use walkdir;

/// Turns a file name into its id in the following steps:
///  - All characters are turned to lowercase
///  - Spaces ` ` are replaced by dashes `-`.
///  - A possible `.md` file extension is removed.
/// ```
///  assert_eq!(name_to_id("Lie Theory.md"), "lie-theory");
///  assert_eq!(name_to_id("Lie Theory"), "lie-theory");
///  assert_eq!(name_to_id("lie-theory"), "lie-theory");
/// ```
pub fn name_to_id(name: &str) -> String {
    name.to_lowercase().replace(" ", "-").replace(".md", "")
}

/// Reads a passed directory recursively, returning a hashmap containing
///  - An entry for every '.md' file in the directory or any subdirectories
///  - The key will be the file name, without the file extension, in lowercase and with spaces replaced by dashes
///  - The value will be an instance of Note containing metadata of the file.
pub fn create_index(directory: &Path) -> color_eyre::Result<HashMap<String, Note>> {
    Ok(walkdir::WalkDir::new(directory)
        .into_iter()
        // Ignore dot-folders and dotfiles
        .filter_entry(is_not_hidden)
        // Check only OKs
        .flatten()
        // Check only markdown files
        .filter(is_markdown)
        // Convert tiles tot notes
        .map(|entry| Note::from_path(entry.path()))
        // Skip errors
        .flatten()
        // Extract name and convert to id
        .map(|note| (name_to_id(&note.name), note))
        // Collect into hash map
        .collect())
}

/// Checks if the given dir entry is 'hidden', i.e. not the root of a search and prefixed by a dot.
fn is_not_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}
/// Checks if the given dir entry is a markdown file, i.e. a file whose name ends in '.md'
fn is_markdown(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_file() && entry.file_name().to_string_lossy().ends_with(".md")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_id_conversion() {
        assert_eq!(name_to_id("Lie Theory.md"), "lie-theory");
        assert_eq!(name_to_id("Lie Theory"), "lie-theory");
        assert_eq!(name_to_id("lie-theory"), "lie-theory");
    }
}
