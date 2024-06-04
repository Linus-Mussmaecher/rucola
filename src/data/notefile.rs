use std::{fs, io::Write, path};

use crate::{config, error};

use super::*;

/// Checks if 'new_name' is a valid new file name, in particular not a path.
/// Then retrieves the note of the given id from the index.
/// Creates a new path from the old path with the new file name.
/// The new extension is the one from the new path if given, if none is given (and no extension is not valid in the config), then the old extension is reapplied.
/// Then moves the old file to the new location and updates the index.
pub fn rename_note_file(
    index: &mut super::NoteIndexContainer,
    id: &str,
    new_name: Option<String>,
    config: &config::Config,
) -> Result<(), error::RucolaError> {
    let index_b = index.borrow_mut();
    // Retrieve the old version from the table
    let note = index_b
        .inner
        .get(id)
        .ok_or_else(|| error::RucolaError::NoteNoteFound(id.to_owned()))?;

    // Remember old path
    let old_path = note.path.clone();

    // Create a path from the input.
    let input_path = path::Path::new(
        new_name
            .as_ref()
            .ok_or_else(|| error::RucolaError::Input("Empty input field.".to_owned()))?,
    );

    // Check that the user hasn't given a full path
    if input_path.components().count() > 1 {
        return Err(error::RucolaError::Input(
            "File name cannot be a path.".to_owned(),
        ));
    }

    // Create a new path by combining the name from the input with the rest of the old path.
    let mut new_path = old_path.clone();
    new_path.set_file_name(
        input_path
            .file_name()
            .ok_or_else(|| error::RucolaError::Input("New name cannot be empty.".to_owned()))?,
    );

    // If this new name has not introduced an extension, re-set the previous one.
    if new_path.extension().is_none() && !config.is_valid_extension("") {
        new_path.set_extension(old_path.extension().unwrap_or_default());
    }

    drop(index_b);

    // Actual move
    move_note_file_inner(id, index, old_path, new_path)?;

    Ok(())
}

pub fn move_note_file(
    index: &mut NoteIndexContainer,
    id: &str,
    new_path_buf: Option<String>,
    config: &config::Config,
) -> Result<(), error::RucolaError> {
    let index_b = index.borrow_mut();
    // Retrieve the old version from the table
    let note = index_b
        .inner
        .get(id)
        .ok_or_else(|| error::RucolaError::NoteNoteFound(id.to_owned()))?;

    // Get input
    let mut new_path_buf =
        new_path_buf.ok_or_else(|| error::RucolaError::Input("Empty input field.".to_owned()))?;

    // If a directory is given, re-use the old name
    if new_path_buf.ends_with("/") {
        new_path_buf.push_str(&note.name);
    }

    // Create a path from the given buffer (handling the parsing of the path).
    // Then extend vault path with given path
    let mut new_path = config.create_vault_path();
    new_path.push(path::Path::new(&new_path_buf));

    // If this has not introduced an extension, and no-extension is not allowed per the config, re-set the previous one.
    if new_path.extension().is_none() && !config.is_valid_extension("") {
        if let Some(old_ext) = note.path.extension() {
            new_path.set_extension(old_ext);
        }
    }
    // If this has still not introduced an extension, ask the config file for a default one.
    config.validate_file_extension(&mut new_path);

    let old_path = note.path.clone();

    drop(index_b);
    // Acutally move the file and update the index
    move_note_file_inner(id, index, old_path, new_path.to_path_buf())
}

/// Moves the file from source to target, if successful removes the old note from the table and inserts a new note with most values copied from the one removed and path, name and index updated to reflect the new path.
fn move_note_file_inner(
    old_id: &str,
    index: &mut NoteIndexContainer,
    source: path::PathBuf,
    target: path::PathBuf,
) -> Result<(), error::RucolaError> {
    // borrow index mutably
    let mut index = index.borrow_mut();

    // create new name and id
    let new_name = target
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let new_id = super::name_to_id(&new_name);

    // actual fs copy (early returns if unsuccessfull)
    fs::rename(source, target.clone())?;

    // Extract old note from the index.
    let mut note = index
        .inner
        .remove(old_id)
        .ok_or_else(|| error::RucolaError::NoteNoteFound(old_id.to_owned()))?;

    // === RENAMING ===
    // Create a regex that find links to the old name or id
    let mut regex_builder = String::new();
    regex_builder.push_str("(\\[\\[)(");
    regex_builder.push_str(&note.name); // at this point, this is still the old name
    regex_builder.push_str("|");
    regex_builder.push_str(old_id);
    regex_builder.push_str(")(\\|?[^\\|^\\]^\\]]*\\]\\])");

    let mut replacement_builder = String::new();
    replacement_builder.push_str("${1}");
    replacement_builder.push_str(&new_name);
    replacement_builder.push_str("${2}");

    let reg = regex::Regex::new(&regex_builder)?;
    for other_note in index
        // search for references to the old id.
        .blinks_vec(old_id)
        .iter()
        .filter_map(|(id, _)| index.inner.get(id))
    {
        let old_content = std::fs::read_to_string(&other_note.path)?;

        let res = reg.replace_all(&old_content, &replacement_builder);

        let file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(&other_note.path)?;
    }

    // Fix the notes values, remembering the old name.
    note.name = new_name;
    note.path = target;

    // Re-insert the fixed note.
    index.inner.insert(new_id, note);

    Ok(())
}

/// Deletes the note of the given id from the index, then follows its path and deletes it in the file system.
pub fn delete_note_file(
    index: &mut NoteIndexContainer,
    id: &str,
) -> Result<(), error::RucolaError> {
    let table = &mut index.borrow_mut().inner;
    // Follow its path and delete it
    fs::remove_file(path::Path::new(
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
    let mut path = config.create_vault_path();
    path.push(input_path.unwrap_or_else(|| "Untitled".to_owned()));

    // If there was no manual extension set, take the default one
    config.validate_file_extension(&mut path);

    // Create the file
    let mut file = fs::File::create(path.clone())?;

    // Write an preliminary input, so the file isn't empty (messed with XDG for some reason).
    write!(
        file,
        "#{}",
        path.file_stem()
            .map(|fs| fs.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_owned())
    )?;

    // Add the file to the index if nothing threw and error and early returned.
    index.borrow_mut().register(path::Path::new(&path));

    Ok(())
}

pub fn create_html(
    note: &Note,
    config: &config::Config,
) -> Result<path::PathBuf, error::RucolaError> {
    // Read content of markdown(plaintext) file
    let content = fs::read_to_string(&note.path)?;

    // Parse markdown into AST
    let arena = comrak::Arena::new();
    let root = comrak::parse_document(
        &arena,
        &content,
        &comrak::Options {
            extension: comrak::ExtensionOptionsBuilder::default()
                .wikilinks_title_after_pipe(true)
                .math_dollars(true)
                .build()
                .unwrap(),
            ..Default::default()
        },
    );

    let mut contains_math = false;

    for node in root.descendants() {
        // correct id urls for wiki links
        match node.data.borrow_mut().value {
            comrak::nodes::NodeValue::WikiLink(ref mut link) => {
                link.url = format!("{}.html", super::name_to_id(&link.url));
            }
            comrak::nodes::NodeValue::Math(ref mut math) => {
                contains_math = true;
                let x = &mut math.literal;
                // re-insert the dollar at beginning and end to make mathjax pick it up
                x.insert(0, '$');
                x.push('$');
                // if display math, do it again.
                if math.display_math {
                    x.insert(0, '$');
                    x.push('$');
                }
                *x = config.perform_replacements(x);
            }
            _ => {}
        }
    }

    // calculate target path
    let mut tar_path = config.create_vault_path();
    tar_path.push(".html/");

    std::fs::create_dir_all(tar_path.clone())?;

    tar_path.set_file_name(format!(".html/{}", super::name_to_id(&note.name)));
    tar_path.set_extension("html");

    let mut tar_file = std::fs::File::create(tar_path.clone())?;

    writeln!(tar_file, "<title>{}</title>", note.name)?;
    config.add_preamble(&mut tar_file, contains_math)?;

    comrak::format_html(
        root,
        &comrak::Options {
            extension: comrak::ExtensionOptionsBuilder::default()
                .wikilinks_title_after_pipe(true)
                .math_dollars(true)
                .build()
                .unwrap(),
            ..Default::default()
        },
        &mut tar_file,
    )?;

    Ok(tar_path)
}

#[cfg(test)]
mod tests {

    use regex::Regex;

    #[test]
    fn test_renaming() {
        // Read content of markdown(plaintext) file
        let content = String::from(
            "Link1 that links to [[Smooth Map|a super-smooth map]].\nLink2 that links to the [[Smooth Map]].\nLink3 that links to the [[smooth-map]].\nLink4 that links to a final [[smooth-map|map that is smooth]].",
        );
        let content_tar = String::from(
            "Link1 that links to [[Atlas of Worlds|a super-smooth map]].\nLink2 that links to the [[Atlas of Worlds]].\nLink3 that links to the [[Atlas of Worlds]].\nLink4 that links to a final [[Atlas of Worlds|map that is smooth]].",
        );

        let link = "Smooth Map";

        let mut s = String::new();
        s.push_str("(\\[\\[)(");
        s.push_str(link);
        s.push_str("|");
        s.push_str(&crate::data::name_to_id(link));
        s.push_str(")(\\|?[^\\|^\\]^\\]]*\\]\\])");

        let reg = Regex::new(&s).unwrap();

        let res = reg.replace_all(&content, "${1}Atlas of Worlds${3}");

        assert_eq!(res.to_string(), content_tar);
    }

    // #[test]
    // fn test_reparsing() {
    //     let index = NoteIndex::new(
    //         std::path::Path::new("./tests/common/notes/"),
    //         &config::Config::default(),
    //     );
    //     assert_eq!(index.inner.len(), 11);

    //     // Open the file.
    //     let mut file = fs::File::open(std::path::Path::new(
    //         "./tests/common/notes/math/Lie Group.md",
    //     ))
    //     .unwrap();

    //     // Read content of markdown(plaintext) file
    //     let mut content = String::new();
    //     std::io::Read::read_to_string(&mut file, &mut content).unwrap();

    //     // Parse markdown into AST
    //     let arena = comrak::Arena::new();
    //     let root = comrak::parse_document(
    //         &arena,
    //         &content,
    //         &comrak::Options {
    //             extension: comrak::ExtensionOptionsBuilder::default()
    //                 .wikilinks_title_after_pipe(true)
    //                 .math_dollars(true)
    //                 .build()
    //                 .unwrap(),
    //             ..Default::default()
    //         },
    //     );
    //     let mut file2 = fs::File::open(std::path::Path::new("./test_sink.md")).unwrap();
    //     let _ = comrak::format_commonmark(
    //         root,
    //         &comrak::Options {
    //             extension: comrak::ExtensionOptionsBuilder::default()
    //                 .wikilinks_title_after_pipe(true)
    //                 .math_dollars(true)
    //                 .build()
    //                 .unwrap(),
    //             ..Default::default()
    //         },
    //         &mut file2,
    //     );

    //     let mut output = Vec::new();
    //     let _ = comrak::format_commonmark(
    //         root,
    //         &comrak::Options {
    //             extension: comrak::ExtensionOptionsBuilder::default()
    //                 .wikilinks_title_after_pipe(true)
    //                 .math_dollars(true)
    //                 .build()
    //                 .unwrap(),
    //             ..Default::default()
    //         },
    //         &mut output,
    //     );
    //     let transformed_content = String::from_utf8(output.clone()).unwrap();
    //     fs::write("./test_sink.md", output).unwrap();

    //     assert_eq!(content, transformed_content);
    // }
}
