use std::{borrow::BorrowMut, collections::HashMap};

use crate::{error, files};

use super::Note;

pub type NoteIndexContainer = std::rc::Rc<std::cell::RefCell<NoteIndex>>;

/// Contains an indexed and hashed list of notes
pub struct NoteIndex {
    /// The wrapped HashMap, available only in the data module.
    pub(super) inner: HashMap<String, Note>,

    /// === Config ===
    /// The file tracker that sends file events and watches the structure of the vault of this index.
    tracker: files::FileTracker,
    /// The HtmlBuilder this index uses to create its HTML files.
    builder: files::HtmlBuilder,
}

impl NoteIndex {
    /// Reads a passed directory recursively, returning a hashmap containing
    ///  - An entry for every '.md' file in the directory or any subdirectories
    ///  - The key will be the file name, without the file extension, in lowercase and with spaces replaced by dashes
    ///  - The value will be an instance of Note containing metadata of the file.
    ///
    /// All IO errors that happeded during the creation or the (potential) HTML conversion are returned alongside.
    pub fn new(
        mut tracker: files::FileTracker,
        builder: files::HtmlBuilder,
    ) -> (Self, Vec<error::RucolaError>) {
        // create an error struct
        let mut errors = vec![];
        // collect all the notes from the vault folder
        let inner = tracker
            .get_walker() // Check only OKs
            .flatten()
            // Convert tiles to notes and skip errors
            .filter(|entry| entry.metadata().is_ok_and(|md| md.is_file()))
            .flat_map(|entry| match Note::from_path(entry.path()) {
                Ok(note) => Some(note),
                Err(e) => {
                    errors.push(e.into());
                    None
                }
            })
            // Extract name and convert to id
            .map(|note| (super::name_to_id(&note.name), note))
            // Collect into hash map
            .collect::<HashMap<_, _>>();

        // create htmls and save errors
        errors.extend(
            inner
                .iter()
                .map(|(_id, note)| builder.create_html(&note, false))
                .flat_map(|res| match res {
                    Ok(_) => None,
                    Err(e) => Some(e),
                }),
        );

        // let the watcher start watching _after_ all htmls have been re-done
        match tracker.initialize_watching() {
            Ok(_) => {}
            Err(e) => errors.push(e.into()),
        };

        (
            Self {
                inner,
                tracker,
                builder,
            },
            errors,
        )
    }

    /// Wrapper of the HashMap::get() Function
    pub fn get(&self, key: &str) -> Option<&Note> {
        self.inner.get(key)
    }

    /// Handle all file events on notes, as found by the contained tracker.
    ///  - Renames and moves are tracked
    ///  - new file creations with in the vault folder are checked for notes and added if appropriate
    ///  - removed files are removed from the index (if they were present)
    ///  - Modifications of files are checked for being notes and if so, the respective index entries are updated with the new data.
    /// Returns wether the index has changed.
    pub fn handle_file_events(&mut self) -> Result<bool, error::RucolaError> {
        let mut modifications = false;
        for event in self.tracker.try_events_iter().flatten() {
            match event.kind {
                notify::EventKind::Create(kind) => {
                    // Creations:
                    // - Check if a file was created (we don't care about folders)
                    // - Check for each path if we are interested in it (gitignore + extensions from config)
                    // - Try to load the note and index it
                    if kind == notify::event::CreateKind::File {
                        for path in event.paths {
                            if self.tracker.is_tracked(&path) {
                                if let Ok(note) = super::Note::from_path(&path) {
                                    // create html on creation
                                    self.builder.create_html(&note, false)?;
                                    // insert the note
                                    self.inner.insert(super::name_to_id(&note.name), note);
                                    modifications = true;
                                }
                            }
                        }
                    }
                }
                notify::EventKind::Modify(kind) => {
                    // Modifications
                    // - For renames, remove all notes at a path coinciding with a rename source, then insert all notes from a rename target.
                    // - For (meta)data modifications, reload the entire note
                    match kind {
                        // Renames and moves
                        notify::event::ModifyKind::Name(mode) => match mode {
                            notify::event::RenameMode::From => {
                                self.inner
                                    .retain(|_, note| !event.paths.contains(&note.path));
                                modifications = true;
                            }
                            notify::event::RenameMode::To => {
                                for path in event.paths {
                                    if self.tracker.is_tracked(&path) {
                                        if let Ok(note) = super::Note::from_path(&path) {
                                            // create html on creation
                                            self.builder.create_html(&note, false)?;
                                            // insert the note from the new location
                                            self.inner.insert(super::name_to_id(&note.name), note);
                                            modifications = true;
                                        }
                                    }
                                }
                            }
                            notify::event::RenameMode::Both => {}
                            notify::event::RenameMode::Any => {}
                            notify::event::RenameMode::Other => {}
                        },
                        // General edits
                        notify::event::ModifyKind::Data(_)
                        | notify::event::ModifyKind::Metadata(_) => {
                            for (_id, note) in self.inner.borrow_mut().iter_mut() {
                                if event.paths.contains(&note.path) {
                                    if let Ok(new_note) = Note::from_path(&note.path) {
                                        // create html on creation
                                        self.builder.create_html(&note, false)?;
                                        // replace the index entry
                                        *note = new_note;
                                        modifications = true;
                                    }
                                }
                            }
                        }
                        notify::event::ModifyKind::Any => {}
                        notify::event::ModifyKind::Other => {}
                    }
                }
                // Remove events: Keep only those notes whose path was not removed
                notify::EventKind::Remove(kind) => {
                    if kind == notify::event::RemoveKind::File {
                        self.inner
                            .retain(|_, note| !event.paths.contains(&note.path));
                        modifications = true;
                    }
                }
                // Do nothing in the other cases
                notify::EventKind::Access(_) => {}
                notify::EventKind::Other => {}
                notify::EventKind::Any => {}
            }
        }
        Ok(modifications)
    }

    /// Returns an iterator over pairs of (id, name) of notes linked from this note.
    pub fn links_vec(&self, source_id: &str) -> Vec<(String, String)> {
        self.inner
            .get(source_id)
            .map(|source| {
                source
                    .links
                    .iter()
                    .flat_map(|link_id| {
                        self.inner
                            .get(link_id)
                            .map(|note| note.name.clone())
                            .map(|name| (link_id.to_owned(), name))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns an iterator over pairs of (id, name) of notes linking to this note.
    pub fn blinks_vec(&self, target_id: &str) -> Vec<(String, String)> {
        let id_copy = target_id.to_string();
        self.inner
            .iter()
            .filter(|(_other_id, note)| note.links.contains(&id_copy))
            .map(|(id, note)| (id.to_owned(), note.name.to_owned()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files;

    #[test]
    fn test_indexing() {
        let config = files::Config::default();
        let tracker = files::FileTracker::new(&config, std::path::PathBuf::from("./tests"));
        let builder = files::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 11);

        assert!(!index.inner.contains_key("booksold"));

        let os = index.inner.get("operating-systems").unwrap();
        let lg = index.inner.get("lie-group").unwrap();
        let ma = index.inner.get("manifold").unwrap();

        assert_eq!(os.links.len(), 6);
        assert_eq!(os.tags, ["#os"]);
        assert_eq!(os.name, "Operating Systems");
        assert_eq!(os.words, 41);

        assert_eq!(lg.links, ["manifold", "smooth-map", "topology"]);
        assert_eq!(ma.tags.len(), 2);
    }

    #[test]
    fn test_links_blinks() {
        let config = files::Config::default();
        let tracker = files::FileTracker::new(&config, std::path::PathBuf::from("./tests"));
        let builder = files::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 11);

        assert_eq!(
            index.links_vec("lie-group"),
            vec![
                ("manifold".to_string(), "Manifold".to_string()),
                ("smooth-map".to_string(), "Smooth Map".to_string()),
                ("topology".to_string(), "Topology".to_string()),
            ]
        );
        assert_eq!(
            index.blinks_vec("lie-group"),
            vec![("manifold".to_string(), "Manifold".to_string()),]
        );
    }
}
