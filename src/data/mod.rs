mod note;

use std::{collections::HashMap, env, fs::File, path::Path};
use walkdir;

pub use note::Note;

/// Reads a passed directory recursively, returning a hashmap containing
///  - An entry for every '.md' file in the directory or any subdirectories
///  - The key will be the file name, without the file extension, in lowercase and with spaces replaced by dashes
///  - The value will be an instance of Note containing metadata of the file.
pub fn create_index(directory: &Path) -> HashMap<String, Note> {
    //TODO: Errors
    println!("Current dir: {:?}", env::current_dir());

    let mut index = HashMap::new();

    for e in walkdir::WalkDir::new(directory).into_iter().flatten() {
        if e.file_type().is_file() && e.file_name().to_string_lossy().ends_with(".md") {
            if let Ok(file) = File::open(e.path()) {
                index.insert(
                    e.file_name()
                        .to_string_lossy()
                        .to_lowercase()
                        .replace(" ", "-")
                        .replace(".md", ""),
                    Note::from_file(file).unwrap(),
                );
            }
        }
    }

    index
}
