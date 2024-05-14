use std::{io::Write, path};

use crate::config;

use super::*;

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
pub fn rename_note_file(
    index: &mut NoteIndexContainer,
    id: &str,
    new_name: Option<String>,
) -> bool {
    let table = &mut index.borrow_mut().inner;
    // Remove the old version from the table
    if let Some(note) = table.get(id) {
        // Remember old path
        let old_path = note.path.clone();
        // Create a new path from the input.
        let mut new_path = old_path.clone();
        new_path.set_file_name(
            new_name
                .unwrap_or_default()
                .split("/")
                .last()
                .unwrap_or("Untitled"),
        );
        // If this new name has not introduced an extension, re-set the previous one.
        if new_path.extension().is_none() {
            if let Some(ext) = old_path.extension() {
                new_path.set_extension(ext);
            }
        }
        // Actual move
        move_note_file_inner(id, table, old_path, new_path)
    } else {
        false
    }
}
pub fn move_note_file(
    index: &mut NoteIndexContainer,
    id: &str,
    new_path: Option<String>,
    config: &config::Config,
) -> bool {
    let table = &mut index.borrow_mut().inner;
    // Remove the old version from the table
    if let Some(note) = table.get(id) {
        // Piece together the new file path
        let mut new_path = if let Some(new_path) = new_path {
            let mut temp_path = config.get_vault_path();
            temp_path.push(new_path);
            temp_path
        } else {
            note.path.clone()
        };
        // If this new name has not introduced an extension, re-set the previous one.
        if new_path.extension().is_none() {
            if let Some(ext) = note.path.extension() {
                new_path.set_extension(ext);
            }
        }
        // Acutally move the file and update the index

        move_note_file_inner(id, table, note.path.clone(), new_path.to_path_buf())
    } else {
        false
    }
}

/// Removes the file with the given id from the table, then moves the file from source to target and inserts a new note with most values copied from the one removed and path, name and index updated to reflect the new path.
fn move_note_file_inner(
    id: &str,
    table: &mut std::collections::HashMap<String, Note>,
    source: path::PathBuf,
    target: path::PathBuf,
) -> bool {
    if let Some(mut note) = table.remove(id) {
        let new_name = target
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let new_id = super::name_to_id(&new_name);

        // acutal fs copy

        if std::fs::rename(source, target.clone()).is_ok() {
            note.name = new_name;
            note.path = target;
            table.insert(new_id, note);
            return true;
        }
    }
    false
}

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
pub fn delete_note_file(index: &mut NoteIndexContainer, id: &str) -> bool {
    // Get the note
    if let Some(note) = index.borrow().get(id) {
        // Follow its path and delete it
        if std::fs::remove_file(path::Path::new(&note.path)).is_ok() {
            // If that both worked, remove it from the index.
            index.borrow_mut().remove(id);
            return true;
        }
    }
    false
}

/// Creates a note of the given name in the file system (relative to the vault) and registers it in the given index.
pub fn create_note_file(
    index: &mut NoteIndexContainer,
    input_path: Option<&String>,
    config: &config::Config,
) -> bool {
    // Piece together the file path
    let mut path = config.get_vault_path();
    path.push(input_path.map(|s| s.as_str()).unwrap_or("Untitled"));
    // If there was no manual extension set, take the default one
    if path.extension().is_none() {
        path.set_extension(config.get_default_extension());
    }
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
