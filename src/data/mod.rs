mod note;
pub use note::Note;

mod note_statistics;
pub use note_statistics::EnvironmentStats;

mod filter;
pub use filter::Filter;

mod index;
pub use index::NoteIndex;
pub use index::NoteIndexContainer;

pub mod notefile;

/// Converts a file path to the name of the file
pub fn path_to_name(path: &std::path::Path) -> String {
    path.file_stem()
        .map(|os| os.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Turns a file name or link into its id in the following steps:
///  - everything after the first # or ., including the # or ., is ignored
///  - All characters are turned to lowercase
///  - Spaces ` ` are replaced by dashes `-`.
///  - A possible file extension is removed.
/// ```
///  assert_eq!(name_to_id("Lie Theory#Definition"), "lie-theory");
///  assert_eq!(name_to_id("Lie Theory.md"), "lie-theory");
///  assert_eq!(name_to_id("Lie Theory"), "lie-theory");
///  assert_eq!(name_to_id("lie-theory"), "lie-theory");
/// ```
pub fn name_to_id(name: &str) -> String {
    name.split(['#', '.'])
        .take(1)
        .collect::<String>()
        .to_lowercase()
        .replace(' ', "-")
        .replace(".md", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_id_conversion() {
        assert_eq!(name_to_id("Lie Theory#Definition"), "lie-theory");
        assert_eq!(name_to_id("Lie Theory.md"), "lie-theory");
        assert_eq!(name_to_id("Lie Theory"), "lie-theory");
        assert_eq!(name_to_id("lie-theory"), "lie-theory");
    }
}
