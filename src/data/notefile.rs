use std::{io::Write, path};

use crate::config;

use super::*;

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
pub fn rename_note_file(index: &mut NoteIndexContainer, id: &str, new_name: String) -> bool {
    let table = &mut index.borrow_mut().inner;
    // Remove the old version from the table
    if let Some(mut note) = table.remove(id) {
        // Remember old path
        let old_path = note.path.clone();
        // Create a new path from the input.
        // This handles splitting into name and extension and gets rid of folders etc. in the path.
        let new_path = path::Path::new(&new_name);
        // Replace the old file name with the new one
        if let Some(name) = new_path.file_name() {
            note.path.set_file_name(name);
        }
        // If this new name has not introduced an extension, re-set the previous one.
        if new_path.extension().is_none() {
            if let Some(ext) = old_path.extension() {
                note.path.set_extension(ext);
            }
        }
        // update its name & id
        note.name = note
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(new_name);
        let new_id = super::name_to_id(&note.name);
        // Actually change values in the file system
        if std::fs::rename(old_path, note.path.clone()).is_ok() {
            // This only happens on success
            // re-insert into map under new index
            table.insert(new_id, note);
        }
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
