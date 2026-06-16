mod note;
pub use note::Note;

mod note_statistics;
pub use note_statistics::EnvironmentStats;
pub use note_statistics::NoteColumn;

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
        .split('/')
        .rev()
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

/// Converts a path to the path where a copy should be written to.
pub fn path_to_copy_path(path: &std::path::Path) -> std::path::PathBuf {
    let file_name = format!(
        "{}",
        chrono::Local::now().format(
            path.file_name()
                .and_then(|osstr| osstr.to_str())
                .unwrap_or("duplicate-%F")
        )
    );

    // Create the new path
    let mut new_path = path.with_file_name(file_name.clone());

    // If that path exists (either the date replacing changed nothing or the file already existed due to a previous copy), prepend `copy_` until it doesn't or up to 10 times.
    let mut tries = 0;
    while new_path.exists() && tries < 10 {
        new_path = new_path.with_file_name(
            "copy_".to_owned()
                + new_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_default(),
        );
        tries += 1;
    }

    new_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_id_conversion() {
        assert_eq!(name_to_id("Lie Theory#Definition"), "lie-theory");
        assert_eq!(name_to_id("Lie Theory.md"), "lie-theory");
        assert_eq!(name_to_id("Lie Theory"), "lie-theory");
        assert_eq!(name_to_id("lie-theory"), "lie-theory");
        assert_eq!(name_to_id("Math/Lie Theory.md"), "lie-theory");
        assert_eq!(name_to_id("Math/Lie Theory"), "lie-theory");
        assert_eq!(name_to_id("Math/Algebra/Lie Theory"), "lie-theory");
        assert_eq!(name_to_id("Math/lie-theory"), "lie-theory");
        assert_eq!(name_to_id("Monthlies/monthly-%m-%Y"), "monthly-%m-%y");
        assert_eq!(name_to_id("Monthlies/monthly-06-26"), "monthly-06-26");
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
