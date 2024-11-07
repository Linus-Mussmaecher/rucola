use std::{borrow::BorrowMut, collections::HashMap};

use itertools::Itertools;

use crate::{error, io};

use super::Note;

/// Contains a NoteIndex and wraps it to provide easy mutable access from different areas of the code.
pub type NoteIndexContainer = std::rc::Rc<std::cell::RefCell<NoteIndex>>;

/// Contains an indexed and hashed list of notes
pub struct NoteIndex {
    /// The wrapped HashMap, available only in the data module.
    pub(super) inner: HashMap<String, Note>,

    /// === Config ===
    /// The file tracker that sends file events and watches the structure of the vault of this index.
    tracker: io::FileTracker,
    /// The HtmlBuilder this index uses to create its HTML files.
    builder: io::HtmlBuilder,
}

impl std::fmt::Debug for NoteIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl NoteIndex {
    /// Reads a passed directory recursively, returning a hashmap containing
    ///  - An entry for every '.md' file in the directory or any subdirectories
    ///  - The key will be the file name, without the file extension, in lowercase and with spaces replaced by dashes
    ///  - The value will be an instance of Note containing metadata of the file.
    ///
    /// All IO errors that happeded during the creation or the (potential) HTML conversion are returned alongside.
    pub fn new(
        mut tracker: io::FileTracker,
        builder: io::HtmlBuilder,
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
                    errors.push(e);
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
                .values()
                .map(|note| builder.create_html(note, false))
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
    ///
    /// Returns wether the index has changed, and a list of all IdChanges.
    pub fn handle_file_events(&mut self) -> error::Result<(bool, Vec<String>)> {
        let mut modifications = false;
        let mut id_changes = vec![];
        for event in self.tracker.try_events_iter().flatten() {
            match event.kind {
                notify::EventKind::Create(_)
                // also trigger on the target of a rename (new location)
                | notify::EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::To,
                ))
                     => {
                    // Creations:
                    // - Check if a file was created (we don't care about folders)
                    // - Check for each path if we are interested in it (gitignore + extensions from config)
                    // - Try to load the note and index it
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
                // Remove events: Keep only those notes whose path was not removed
                notify::EventKind::Remove(_) 
                // also trigger on the source of a renamed file (former location)
                | notify::EventKind::Modify(notify::event::ModifyKind::Name(
                    notify::event::RenameMode::From,
                ))
                => {
                    let deleted_path = event
                        .paths
                        .first()
                        .ok_or_else(|| error::RucolaError::NotifyEventError(event.clone()))?.to_owned();
                    let deleted_path = deleted_path
                        .canonicalize()
                        .unwrap_or(deleted_path);

                    if let Some(old_id) = self
                        .inner
                        .iter()
                        .find(|(_id, note)| note.path.to_path_buf() == *deleted_path)
                        .map(|(id, _n)| id.to_owned())
                    {
                        self.inner.remove(&old_id);
                        modifications = true;
                        id_changes.push(old_id);
                    }
                }
                notify::EventKind::Modify(_kind) => {
                    // Modifications
                    // - For modifications, reload the entire note
                            for (_id, note) in self.inner.borrow_mut().iter_mut() {
                                if event.paths.iter().flat_map(|path| path.canonicalize()).contains(&note.path) {
                                    if let Ok(new_note) = Note::from_path(&note.path) {
                                        // create html on creation
                                        self.builder.create_html(&new_note, false)?;
                                        // replace the index entry
                                        *note = new_note;
                                        modifications = true;
                                    }
                                }
                            }
                }
                // Do nothing in the other cases
                notify::EventKind::Access(_) => {}
                notify::EventKind::Other => {}
                notify::EventKind::Any => {}
            }
        }
        // just to be sure
        modifications |= !id_changes.is_empty();
        Ok((modifications, id_changes))
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
                    .unique()
                    .sorted_by(|(id1, _), (id2, _)| id1.cmp(id2))
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
            .unique()
            .sorted_by(|(id1, _), (id2, _)| id1.cmp(id2))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io;

    #[test]
    fn test_index() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
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
    fn test_links() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
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
            index.links_vec("atlas"),
            vec![
                ("chart".to_string(), "Chart".to_string()),
                ("manifold".to_string(), "Manifold".to_string()),
                ("topology".to_string(), "Topology".to_string()),
            ]
        );
    }

    #[test]
    fn test_blinks() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 11);

        assert_eq!(
            index.blinks_vec("lie-group"),
            vec![("manifold".to_string(), "Manifold".to_string())]
        );

        assert_eq!(
            index.blinks_vec("manifold"),
            vec![
                ("atlas".to_string(), "Atlas".to_string()),
                ("chart".to_string(), "Chart".to_string()),
                ("lie-group".to_string(), "Lie Group".to_string()),
                ("smooth-map".to_string(), "Smooth Map".to_string()),
            ]
        );
    }
}
