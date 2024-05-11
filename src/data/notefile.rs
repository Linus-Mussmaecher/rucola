use std::{io::Write, path};

use crate::config;

use super::*;

pub fn delete_note_file(index: &mut NoteIndexContainer, id: &str) -> bool {
    if if let Some(note) = index.borrow().get(id) {
        std::fs::remove_file(path::Path::new(&note.path)).is_ok()
    } else {
        false
    } {
        index.borrow_mut().remove(id);
        true
    } else {
        false
    }
}

pub fn create_note_file(
    index: &mut NoteIndexContainer,
    name: Option<&String>,
    config: &config::Config,
) -> bool {
    // Piece together the file path
    let mut path = config.get_vault_path();
    path.push(
        name.map(|s| s.trim_start_matches("/"))
            .unwrap_or("Untitled"),
    );
    path.set_extension("md");
    // Create the file
    let file = std::fs::File::create(path.clone());
    if let Ok(mut file) = file {
        let _ = write!(
            file,
            "#{}",
            path.file_stem()
                .map(|fs| fs.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_owned())
        );
    }
    // Add the file to the index
    index.borrow_mut().register(&path::Path::new(&path));
    true
}
