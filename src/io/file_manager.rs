use crate::{data, error};
use std::{fs, io::Write, path};

/// Saves configurations to manipulate the file system the notes are stored in.
#[derive(Debug, Clone)]
pub struct FileManager {
    /// Path to the vault to index.
    vault_path: path::PathBuf,
    /// Default file ending for newly created notes
    default_extension: String,
    /// The editor to use for notes
    editor: Option<String>,
}
impl Default for FileManager {
    fn default() -> Self {
        Self::new(
            &crate::Config::default(),
            std::env::current_dir().expect("Current directory to exist and be accessible."),
        )
    }
}

impl FileManager {
    pub fn new(config: &crate::Config, vault_path: path::PathBuf) -> Self {
        Self {
            vault_path,
            default_extension: config.default_extension.clone(),
            editor: config.editor.clone(),
        }
    }

    /// Takes in a PathBuf and, if the current file extension is not set, append the default one.
    pub fn ensure_file_extension(&self, path: &mut path::PathBuf) {
        if path.extension().is_none() {
            path.set_extension(&self.default_extension);
        }
    }

    /// Checks if 'new_name' is a valid new file name, in particular not a path.
    /// Then retrieves the note of the given id from the index.
    /// Creates a new path from the old path with the new file name.
    /// The new extension is the one from the new path if given, if none is given (and no extension is not valid in the config), then the old extension is reapplied.
    /// Then moves the old file to the new location and updates the index.
    pub fn rename_note_file(
        &self,
        index: &mut data::NoteIndexContainer,
        id: &str,
        new_name: String,
    ) -> error::Result<()> {
        let index_b = index.borrow_mut();
        // Retrieve the old version from the table
        let note = index_b
            .get(id)
            .ok_or_else(|| error::RucolaError::NoteNotFound(id.to_owned()))?;

        // Remember old path
        let old_path = note.path.clone();

        // Create a path from the input.
        let input_path = path::Path::new(&new_name);

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
        if new_path.extension().is_none() {
            new_path.set_extension(old_path.extension().unwrap_or_default());
        }

        // make sure the mutable borrow from the first line in the function is dropped
        drop(index_b);

        // Actual move
        self.move_note_file_inner(id, index, old_path, new_path)
    }

    pub fn move_note_file(
        &self,
        index: &mut data::NoteIndexContainer,
        id: &str,
        mut new_path_buf: String,
    ) -> error::Result<()> {
        let index_b = index.borrow_mut();
        // Retrieve the old version from the table
        let note = index_b
            .get(id)
            .ok_or_else(|| error::RucolaError::NoteNotFound(id.to_owned()))?;

        // If a directory is given, re-use the old name
        if new_path_buf.ends_with("/") {
            new_path_buf.push_str(&note.name);
        }

        // Create a path from the given buffer (handling the parsing of the path).
        // Then extend vault path with given path
        let mut new_path = self.vault_path.clone();
        new_path.push(path::Path::new(&new_path_buf));

        // If this has not introduced an extension, re-set the previous one.
        if new_path.extension().is_none() {
            if let Some(old_ext) = note.path.extension() {
                new_path.set_extension(old_ext);
            }
        }
        // If this has still not introduced an extension, ask the config file for a default one.
        self.ensure_file_extension(&mut new_path);

        let old_path = note.path.clone();

        // make sure the mutable borrow from the first line in this function is dropped
        drop(index_b);
        // Acutally move the file and update the index
        self.move_note_file_inner(id, index, old_path, new_path.to_path_buf())
    }

    /// Moves the file from source to target, if successful removes the old note from the table and inserts a new note with most values copied from the one removed and path, name and index updated to reflect the new path.
    fn move_note_file_inner(
        &self,
        old_id: &str,
        index: &mut data::NoteIndexContainer,
        source: path::PathBuf,
        target: path::PathBuf,
    ) -> error::Result<()> {
        // borrow index mutably
        let index = index.borrow();

        // create new name and id
        let new_name = target
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        // actual fs copy (early returns if unsuccessfull)
        fs::rename(source, target.clone())?;

        // Extract old note from the index.
        let note = index
            .get(old_id)
            .ok_or_else(|| error::RucolaError::NoteNotFound(old_id.to_owned()))?;

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
        replacement_builder.push_str("${3}");

        let reg = regex::Regex::new(&regex_builder)?;
        for other_note in index
            // search for references to the old id.
            .blinks_vec(old_id)
            .iter()
            .filter_map(|(id, _)| index.get(id))
        {
            // open the file once to read its old content
            let old_content = std::fs::read_to_string(&other_note.path)?;

            let res = reg.replace_all(&old_content, &replacement_builder);

            // open the file again
            let mut file = std::fs::OpenOptions::new()
                // this truncate is neccesary to remove the old content
                .truncate(true)
                // standard read-write permissions
                .write(true)
                .read(true)
                .open(&other_note.path)?;
            // write new new (mostly old) string into the file
            file.write_all(res.as_bytes())?;
        }

        Ok(())
    }

    /// Follows a notes path and deletes it in the file system.
    /// Deletion from the index is handled centrally by the file watcher of the index itself.
    pub fn delete_note_file(
        &self,
        index: &mut data::NoteIndexContainer,
        id: &str,
    ) -> error::Result<()> {
        // Follow its path and delete it
        fs::remove_file(path::Path::new(
            // get the note
            &index
                .borrow_mut()
                .get(id)
                .ok_or_else(|| error::RucolaError::NoteNotFound(id.to_owned()))?
                .path,
        ))?;
        Ok(())
    }

    /// Creates a note of the given name in the file system (relative to the vault).
    /// Registration in the index is handled centrally by the file watcher of the index itself.
    pub fn create_note_file(&self, input_path: Option<String>) -> error::Result<()> {
        // Piece together the file path
        let mut path = self.vault_path.clone();
        path.push(input_path.unwrap_or_else(|| "Untitled".to_owned()));

        // If there was no manual extension set, take the default one
        self.ensure_file_extension(&mut path);

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

        Ok(())
    }

    /// Attempts to create a command to open the file at the given path to edit it.
    /// Target should be a markdown file.
    /// Checks:
    ///  - The config file
    ///  - The $EDITOR environment variable
    ///  - the systems default programms
    /// for an applicable program.
    pub fn create_edit_command(
        &self,
        path: &path::PathBuf,
    ) -> error::Result<std::process::Command> {
        // take the editor from the config file
        self.editor
            .as_ref()
            // create a command from it
            .map(|editor_string| open::with_command(path, editor_string))
            // Try the $EDITOR variable
            .or_else(|| {
                std::env::var("EDITOR")
                    .ok()
                    .map(|editor| open::with_command(path, editor))
            })
            // if it was not there, take the default command
            .or_else(|| open::commands(path).pop())
            // if it was also not there, throw an error
            .ok_or_else(|| error::RucolaError::ApplicationMissing)
    }
}
#[cfg(test)]
mod tests {

    #[test]
    fn test_opening() {
        let editor = std::env::var("EDITOR");

        let config = crate::Config::default();
        let fm = super::FileManager::new(&config, std::path::PathBuf::from("./tests"));
        let path = std::path::Path::new("./tests/common/notes/Books.md");

        if let Ok(_editor) = editor {
            // if we can unwrap the env variable, then we should be able to create a command
            fm.create_edit_command(&path.to_path_buf()).unwrap();
        }
    }

    #[test]
    fn test_file_endings() {
        let md_ending_tar = std::path::PathBuf::from("./tests/common/test.md");
        let txt_ending_tar = std::path::PathBuf::from("./tests/common/test.txt");

        let config = crate::Config::default();
        let fm = super::FileManager::new(&config, std::path::PathBuf::from("./tests"));

        let mut no_ending = std::path::PathBuf::from("./tests/common/test");
        let mut md_ending = std::path::PathBuf::from("./tests/common/test.md");
        let mut txt_ending = std::path::PathBuf::from("./tests/common/test.txt");

        fm.ensure_file_extension(&mut no_ending);
        fm.ensure_file_extension(&mut md_ending);
        fm.ensure_file_extension(&mut txt_ending);

        assert_eq!(no_ending, md_ending_tar);
        assert_eq!(md_ending, md_ending_tar);
        assert_eq!(txt_ending, txt_ending_tar);
    }
}
