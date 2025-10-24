mod note;
pub use note::Note;

mod note_statistics;
pub use note_statistics::EnvironmentStats;
pub use note_statistics::SortingMode;

mod filter;
pub use filter::Filter;
pub use filter::TagMatch;

mod index;
pub use index::NoteIndex;
pub use index::NoteIndexContainer;

use unicode_normalization::UnicodeNormalization;

use crate::error;

/// Turns a file name or link into its id in the following steps:
///  - normalize the unicode characters into their composed forms
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
    name.nfc()
        .collect::<String>()
        .split(['#', '.'])
        .take(1)
        .collect::<String>()
        .to_lowercase()
        .replace(' ', "-")
        .replace(".md", "")
}

/// Converts a path to the name of the file, removing the file extension.
pub fn path_to_name(path: &std::path::Path) -> Result<String, error::RucolaError> {
    path.file_stem()
        .map(|os| os.to_string_lossy().to_string())
        .ok_or_else(|| error::RucolaError::NoteNameCannotBeRead(path.to_path_buf()))
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

    #[test]
    fn test_id_conversion_unicode() {
        // Composed form "ö".
        let nfc_o = "\u{00F6}";
        // Decomposed form "ö".
        let nfd_o = "o\u{0308}";

        assert_ne!(nfc_o, nfd_o);
        assert_ne!(format!("K{}rper", nfc_o), format!("K{}rper", nfd_o));

        assert_eq!(name_to_id(nfc_o), name_to_id(nfd_o));
        assert_eq!(
            name_to_id(&format!("K{}rper", nfc_o)),
            name_to_id(&format!("K{}rper", nfd_o))
        );
        assert_eq!(
            name_to_id(&format!("K{}rper.md", nfc_o)),
            name_to_id(&format!("K{}rper#Definition", nfd_o))
        );
        assert_eq!(
            name_to_id(&format!("K{}rper.md", nfc_o)),
            name_to_id(&format!("k{}rper", nfd_o))
        );
    }

    #[test]
    fn test_path_name_conversion() {
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math/Lie-Theory.md")).unwrap(),
            "Lie-Theory"
        );
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math-Stuff/lie-theory.md")).unwrap(),
            "lie-theory"
        );
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math/Lie Theory.md")).unwrap(),
            "Lie Theory"
        );
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math-Stuff/Lie Theory.md")).unwrap(),
            "Lie Theory"
        );
    }

    #[test]
    fn test_path_name_conversion_unicode() {
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math/Körper.md")).unwrap(),
            "Körper"
        );
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math-Stuff/körper.md")).unwrap(),
            "körper"
        );
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math/Maß.md")).unwrap(),
            "Maß"
        );
        assert_eq!(
            path_to_name(std::path::Path::new("General/Math-Stuff/maß-einheiten.md")).unwrap(),
            "maß-einheiten"
        );
    }
}
