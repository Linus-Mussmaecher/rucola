use std::{io::Write, path};

use crate::config;

use super::*;

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
pub fn rename_note_file(index: &mut NoteIndexContainer, id: &str, new_name: String) -> bool {
    let table = &mut index.borrow_mut().inner;
    // Remove the old version from the table
    if let Some(mut note) = table.remove(id) {
        let new_id = super::name_to_id(&new_name);
        // update its name
        note.name = new_name;
        // re-insert under new index
        table.insert(new_id, note);
        // TODO: Update links
        // TODO: Actually change the file and path
        true
    } else {
        false
    }
}

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
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

/// Creates a note of the given name in the file system (relative to the vault) and registers it in the given index.
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
    path.set_extension(config.get_default_ending());
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
