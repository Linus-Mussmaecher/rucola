use std::{io::Write, path};

use crate::{config, error};

use super::*;

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
pub fn rename_note_file(
    index: &mut NoteIndexContainer,
    id: &str,
    new_name: Option<String>,
    config: &config::Config,
) -> Result<(), error::RucolaError> {
    let table = &mut index.borrow_mut().inner;
    // Retrieve the old version from the table
    let note = table
        .get(id)
        .ok_or_else(|| error::RucolaError::NoteNoteFound(id.to_owned()))?;
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
    if new_path.extension().is_none() && !config.is_valid_extension("") {
        if let Some(ext) = old_path.extension() {
            new_path.set_extension(ext);
        }
    }
    // Actual move
    move_note_file_inner(id, table, old_path, new_path)
}
pub fn move_note_file(
    index: &mut NoteIndexContainer,
    id: &str,
    new_path: Option<String>,
    config: &config::Config,
) -> Result<(), error::RucolaError> {
    let table = &mut index.borrow_mut().inner;
    // Retrieve the old version from the table
    let note = table
        .get(id)
        .ok_or_else(|| error::RucolaError::NoteNoteFound(id.to_owned()))?;
    // Piece together the new file path
    let mut new_path = if let Some(new_path) = new_path {
        let mut temp_path = config.get_vault_path();
        temp_path.push(new_path);
        temp_path
    } else {
        note.path.clone()
    };
    // If this has not introduced a file name, re-use the previous one
    if new_path.file_name().is_none() {
        if let Some(name) = note.path.file_name() {
            new_path.set_file_name(name);
        }
    }
    // If this new name has not introduced an extension, and no-extension is not allowed per the config, re-set the previous one.
    if new_path.extension().is_none() && !config.is_valid_extension("") {
        if let Some(ext) = note.path.extension() {
            new_path.set_extension(ext);
        }
    }

    // Acutally move the file and update the index
    move_note_file_inner(id, table, note.path.clone(), new_path.to_path_buf())
}

/// Moves the file from source to target, if successful removes the old note from the table and inserts a new note with most values copied from the one removed and path, name and index updated to reflect the new path.
fn move_note_file_inner(
    id: &str,
    table: &mut std::collections::HashMap<String, Note>,
    source: path::PathBuf,
    target: path::PathBuf,
) -> Result<(), error::RucolaError> {
    // create new name and id
    let new_name = target
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let new_id = super::name_to_id(&new_name);

    // acutal fs copy (early returns if unsuccessfull)
    std::fs::rename(source, target.clone())?;

    // If successful, remove old note, update it and re-insert at new id
    let mut note = table
        .remove(id)
        .ok_or_else(|| error::RucolaError::NoteNoteFound(id.to_owned()))?;

    note.name = new_name;
    note.path = target;
    table.insert(new_id, note);

    Ok(())
}

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
pub fn delete_note_file(
    index: &mut NoteIndexContainer,
    id: &str,
) -> Result<(), error::RucolaError> {
    let table = &mut index.borrow_mut().inner;
    // Follow its path and delete it
    std::fs::remove_file(path::Path::new(
        // get the note
        &table
            .get(id)
            .ok_or_else(|| error::RucolaError::NoteNoteFound(id.to_owned()))?
            .path,
    ))?;
    // If that both worked, remove it from the index.
    table.remove(id);
    Ok(())
}

/// Creates a note of the given name in the file system (relative to the vault) and registers it in the given index.
pub fn create_note_file(
    index: &mut NoteIndexContainer,
    input_path: Option<String>,
    config: &config::Config,
) -> Result<(), error::RucolaError> {
    // Piece together the file path
    let mut path = config.get_vault_path();
    path.push(input_path.unwrap_or_else(|| "Untitled".to_owned()));
    // If there was no manual extension set, take the default one
    config.validate_file_extension(&mut path);
    // Create the file
    let mut file = std::fs::File::create(path.clone())?;
    write!(
        file,
        "#{}",
        path.file_stem()
            .map(|fs| fs.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_owned())
    )?;
    // Add the file to the index if nothing threw and error and early returned.
    index.borrow_mut().register(&path::Path::new(&path));

    Ok(())
}
