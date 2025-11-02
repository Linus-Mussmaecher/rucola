use std::path;
use std::sync::mpsc::{self, TryIter};

use itertools::Itertools;
use notify::Watcher;

use crate::error;

/// Stores configuration to track the file system the notes are stored in.
#[derive(Debug)]
pub struct FileTracker {
    /// Path to the vault to index.
    vault_path: path::PathBuf,
    /// File types to consider notes
    file_types: ignore::types::Types,
    /// Watcher that checks for file changes in the vault directory and needs to be kept alive with this index.
    /// Can be unused because it is just here for RAII.
    #[allow(unused)]
    #[cfg(not(target_os = "macos"))]
    watcher: notify::RecommendedWatcher,
    /// The recommended watcher for macos does not register file renames, so use the PollWatcher fallback instead.
    #[allow(unused)]
    #[cfg(target_os = "macos")]
    watcher: notify::PollWatcher,
    /// Channel from which file change events in the vault directory are deposited by the watcher and can be requested.
    file_change_channel: mpsc::Receiver<Result<notify::Event, notify::Error>>,
}
impl Default for FileTracker {
    fn default() -> Self {
        Self::new(
            &crate::Config::default(),
            std::env::current_dir().expect("Current directory to exist and be accessible."),
        )
        .expect("Watcher to be created and pre-defined file types to work.")
    }
}

impl FileTracker {
    pub fn new(config: &crate::Config, vault_path: path::PathBuf) -> error::Result<Self> {
        // Pre-calculate allowed file types
        let mut types_builder = ignore::types::TypesBuilder::new();
        types_builder.add_defaults();
        for name in config.file_types.iter() {
            types_builder.select(name);
        }

        // Create asynchronous channel for file events.
        let (sender, receiver) = mpsc::channel();

        // Create watcher so we can store it in the file, delaying its drop (which stops its function) until the end of the lifetime of this index.
        #[cfg(not(target_os = "macos"))]
        let watcher = {
            notify::recommended_watcher(move |res| {
                // ignore errors
                let _ = sender.send(res);
            })?
        };

        #[cfg(target_os = "macos")]
        let watcher = {
            // Watcher should poll for external changes every 2 seconds.
            let watcher_config =
                notify::Config::default().with_poll_interval(std::time::Duration::from_secs(2));

            notify::PollWatcher::new(
                move |res| {
                    // ignore errors
                    let _ = sender.send(res);
                },
                watcher_config,
            )?
        };

        Ok(Self {
            vault_path,
            file_types: types_builder.build()?,
            watcher,
            file_change_channel: receiver,
        })
    }

    /// Start watching the vault path.
    /// This action is delayed until now so the watcher is not active while the initial indexing creates a ton of HTML files, which would trigger a ton of file events and a significant hangup.
    pub fn initialize_watching(&mut self) -> Result<(), notify::Error> {
        self.watcher.watch(
            self.vault_path
                .canonicalize()
                .as_ref()
                .unwrap_or(&self.vault_path)
                .as_path(),
            notify::RecursiveMode::Recursive,
        )
    }

    /// Returns a file walker that iterates over all notes to index.
    pub fn get_walker(&self) -> ignore::Walk {
        ignore::WalkBuilder::new(&self.vault_path)
            .types(self.file_types.clone())
            .build()
    }

    /// Wether the given path is supposed to be tracked by rucola or not.
    /// Checks for file endings and gitignore
    pub fn is_tracked(&self, path: &path::Path) -> bool {
        path.canonicalize().is_ok_and(|canon_path| {
            self.get_walker()
                .flatten()
                .flat_map(|dir_entry| dir_entry.path().to_path_buf().canonicalize())
                .contains(&canon_path)
        })
    }

    /// Returns an iterator over all events found by this tracker since the last check.
    pub fn try_events_iter(&self) -> TryIter<'_, Result<notify::Event, notify::Error>> {
        self.file_change_channel.try_iter()
    }

    /// Syncs the tracker right now with the files its tracking
    pub fn poll_file_system(&self) {
        #[cfg(target_os = "macos")]
        {
            self.watcher.poll();
        }
    }
}
#[cfg(test)]
mod tests {

    use std::path;

    #[test]
    fn test_tracker_basic() {
        let no_ending = path::PathBuf::from("./tests/common/notes/Booksold");
        let md = path::PathBuf::from("./tests/common/notes/Books.md");
        let txt = path::PathBuf::from("./tests/common/notes/Books.txt");
        let rs = path::PathBuf::from("./tests/common/notes/Books.rs");

        let config = crate::Config::default();

        let tracker = super::FileTracker::new(&config, path::PathBuf::from("./tests/")).unwrap();

        assert!(!tracker.is_tracked(&no_ending));
        assert!(tracker.is_tracked(&md));
        assert!(!tracker.is_tracked(&txt));
        assert!(!tracker.is_tracked(&rs));
    }

    #[test]
    fn test_tracker_ignored() {
        let md_ignored = path::PathBuf::from("./tests/.html/books.md");
        let html_ignored = path::PathBuf::from("./tests/.html/books.html");

        let config = crate::Config::default();

        let tracker = super::FileTracker::new(&config, path::PathBuf::from("./tests/")).unwrap();

        assert!(!tracker.is_tracked(&md_ignored));
        assert!(!tracker.is_tracked(&html_ignored));
    }

    #[test]
    fn test_tracker_foreign() {
        let md = path::PathBuf::from("./tests/common/notes/Books.md");
        let md_foreign = path::PathBuf::from("./README.md");

        let config = crate::Config::default();

        let tracker = super::FileTracker::new(&config, path::PathBuf::from("./tests/")).unwrap();

        assert!(tracker.is_tracked(&md));
        assert!(!tracker.is_tracked(&md_foreign));
    }
    #[test]
    fn test_tracker_txt() {
        let no_ending = path::PathBuf::from("./tests/common/notes/Booksold");
        let md = path::PathBuf::from("./tests/common/notes/Books.md");
        let txt = path::PathBuf::from("./tests/common/notes/Books.txt");
        let rs = path::PathBuf::from("./tests/common/notes/Books.rs");

        let tracker = super::FileTracker::new(
            &crate::Config {
                file_types: vec!["md".to_owned(), "txt".to_owned()],
                ..Default::default()
            },
            path::PathBuf::from("./tests"),
        )
        .unwrap();

        assert!(!tracker.is_tracked(&no_ending));
        assert!(tracker.is_tracked(&md));
        assert!(tracker.is_tracked(&txt));
        assert!(!tracker.is_tracked(&rs));
    }

    #[test]
    fn test_tracker_all() {
        let no_ending = path::PathBuf::from("./tests/common/notes/Booksold");
        let md = path::PathBuf::from("./tests/common/notes/Books.md");
        let txt = path::PathBuf::from("./tests/common/notes/Books.txt");
        let rs = path::PathBuf::from("./tests/common/notes/Books.rs");

        let tracker = super::FileTracker::new(
            &crate::Config {
                file_types: vec!["all".to_owned()],
                ..Default::default()
            },
            path::PathBuf::from("./tests"),
        )
        .unwrap();

        assert!(!tracker.is_tracked(&no_ending));
        assert!(tracker.is_tracked(&md));
        assert!(tracker.is_tracked(&txt));
        assert!(tracker.is_tracked(&rs));
    }

    // #[test]
    // fn test_watcher_create() {
    //     let tmp = testdir::testdir!();

    //     let config = crate::Config::default();
    //     let fm = crate::io::FileManager::new(&config, tmp.clone());
    //     let tracker = crate::io::FileTracker::new(&config, tmp.clone()).unwrap();
    //     let builder = crate::io::HtmlBuilder::new(&config, tmp.clone());
    //     let mut index = crate::data::NoteIndex::new(tracker, builder).0;

    //     assert!(index.get("atlas").is_none());
    //     assert!(index.get("lie-group").is_none());

    //     fm.create_note_file("Lie Group").unwrap();

    //     let (modifications, id_changes) = index.handle_file_events().unwrap();

    //     assert!(modifications);
    //     assert!(id_changes.is_empty());

    //     assert!(index.get("atlas").is_none());
    //     assert!(index.get("lie-group").is_some());

    //     fm.create_note_file("Math/Atlas").unwrap();

    //     let (modifications, id_changes) = index.handle_file_events().unwrap();

    //     assert!(modifications);
    //     assert!(id_changes.is_empty());

    //     assert!(index.get("atlas").is_some());
    //     assert!(index.get("lie-group").is_some());
    // }

    #[test]
    fn test_watcher_rename() {
        let tmp = testdir::testdir!();

        let config = crate::Config::default();
        let fm = crate::io::FileManager::new(&config, tmp.clone());
        fm.create_note_file("Lie Group").unwrap();
        fm.create_note_file("Math/Atlas").unwrap();

        let tracker = crate::io::FileTracker::new(&config, tmp.clone()).unwrap();
        let builder = crate::io::HtmlBuilder::new(&config, tmp.clone());
        let index = crate::data::NoteIndex::new(
            tracker,
            builder,
            &config,
            std::path::PathBuf::from("./tests"),
        )
        .0;
        let index_con = std::rc::Rc::new(std::cell::RefCell::new(index));

        assert!(index_con.borrow().get("atlas").is_some());
        assert!(index_con.borrow().get("lie-group").is_some());
        assert!(index_con.borrow().get("atlantis").is_none());
        assert!(index_con.borrow().get("lie-soup").is_none());

        fm.rename_note_file(index_con.clone(), "atlas", String::from("Atlantis"))
            .unwrap();
        fm.rename_note_file(index_con.clone(), "lie-group", String::from("Lie Soup"))
            .unwrap();

        let (modifications, mut id_changes) = index_con.borrow_mut().handle_file_events().unwrap();
        id_changes.sort_unstable();

        assert!(modifications);
        assert_eq!(
            id_changes,
            vec![String::from("atlas"), String::from("lie-group"),]
        );

        assert!(index_con.borrow().get("atlas").is_none());
        assert!(index_con.borrow().get("lie-group").is_none());
        assert!(index_con.borrow().get("atlantis").is_some());
        assert!(index_con.borrow().get("lie-soup").is_some());

        let at = index_con.borrow().get("atlantis").unwrap().clone();
        let lg = index_con.borrow().get("lie-soup").unwrap().clone();

        assert_eq!(at.name, String::from("Atlantis"));
        assert_eq!(lg.name, String::from("Lie Soup"));

        assert_eq!(
            at.path,
            tmp.join(path::PathBuf::from("Math"))
                .join(path::PathBuf::from("Atlantis.md"))
        );
        assert_eq!(lg.path, tmp.join(path::PathBuf::from("Lie Soup.md")));
    }

    #[test]
    fn test_watcher_rename_with_delay() {
        let tmp = testdir::testdir!();

        let config = crate::Config::default();
        let fm = crate::io::FileManager::new(&config, tmp.clone());
        fm.create_note_file("Lie Group").unwrap();
        fm.create_note_file("Math/Atlas").unwrap();

        let tracker = crate::io::FileTracker::new(&config, tmp.clone()).unwrap();
        let builder = crate::io::HtmlBuilder::new(&config, tmp.clone());
        let index = crate::data::NoteIndex::new(
            tracker,
            builder,
            &config,
            std::path::PathBuf::from("./tests"),
        )
        .0;
        let index_con = std::rc::Rc::new(std::cell::RefCell::new(index));

        assert!(index_con.borrow().get("atlas").is_some());
        assert!(index_con.borrow().get("lie-group").is_some());
        assert!(index_con.borrow().get("atlantis").is_none());
        assert!(index_con.borrow().get("lie-soup").is_none());

        fm.rename_note_file(index_con.clone(), "atlas", String::from("Atlantis"))
            .unwrap();
        fm.rename_note_file(index_con.clone(), "lie-group", String::from("Lie Soup"))
            .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));

        let (modifications, mut id_changes) = index_con.borrow_mut().handle_file_events().unwrap();
        id_changes.sort_unstable();

        assert!(modifications);
        assert_eq!(
            id_changes,
            vec![String::from("atlas"), String::from("lie-group"),]
        );

        assert!(index_con.borrow().get("atlas").is_none());
        assert!(index_con.borrow().get("lie-group").is_none());
        assert!(index_con.borrow().get("atlantis").is_some());
        assert!(index_con.borrow().get("lie-soup").is_some());

        let at = index_con.borrow().get("atlantis").unwrap().clone();
        let lg = index_con.borrow().get("lie-soup").unwrap().clone();

        assert_eq!(at.name, String::from("Atlantis"));
        assert_eq!(lg.name, String::from("Lie Soup"));

        assert_eq!(
            at.path,
            tmp.join(path::PathBuf::from("Math"))
                .join(path::PathBuf::from("Atlantis.md"))
        );
        assert_eq!(lg.path, tmp.join(path::PathBuf::from("Lie Soup.md")));
    }

    // #[test]
    // fn test_watcher_move() {
    //     let tmp = testdir::testdir!();

    //     let config = crate::Config::default();
    //     let fm = crate::io::FileManager::new(&config, tmp.clone());
    //     fm.create_note_file("Lie Group").unwrap();
    //     fm.create_note_file("Math/Atlas").unwrap();

    //     let tracker = crate::io::FileTracker::new(&config, tmp.clone()).unwrap();
    //     let builder = crate::io::HtmlBuilder::new(&config, tmp.clone());
    //     let index = crate::data::NoteIndex::new(tracker, builder).0;
    //     let mut index_con = std::rc::Rc::new(std::cell::RefCell::new(index));

    //     assert!(index_con.borrow().get("atlas").is_some());
    //     assert!(index_con.borrow().get("lie-group").is_some());

    //     fm.move_note_file(&mut index_con, "atlas", String::from("Topology/"))
    //         .unwrap();
    //     fm.move_note_file(&mut index_con, "lie-group", String::from("Math/Topology/"))
    //         .unwrap();

    //     let (modifications, mut id_changes) = index_con.borrow_mut().handle_file_events().unwrap();

    //     id_changes.sort_unstable_by(|(a1, _b1), (a2, _b2)| a1.cmp(a2));

    //     assert!(modifications);
    //     assert!(id_changes.is_empty());

    //     assert!(index_con.borrow().get("atlas").is_some());
    //     assert!(index_con.borrow().get("lie-group").is_some());

    //     let at = index_con.borrow().get("atlas").unwrap().clone();
    //     let lg = index_con.borrow().get("lie-group").unwrap().clone();

    //     assert_eq!(at.name, String::from("Atlas"));
    //     assert_eq!(lg.name, String::from("Lie Group"));

    //     assert_eq!(
    //         at.path,
    //         tmp.join(&path::PathBuf::from("Math"))
    //             .join(&path::PathBuf::from("Topology"))
    //             .join(&path::PathBuf::from("Atlas.md"))
    //     );
    //     assert_eq!(
    //         lg.path,
    //         tmp.join(&path::PathBuf::from("Topology"))
    //             .join(&path::PathBuf::from("Lie Group.md"))
    //     );
    // }

    // #[test]
    // fn test_watcher_delete() {
    //     let tmp = testdir::testdir!();

    //     let config = crate::Config::default();
    //     let fm = crate::io::FileManager::new(&config, tmp.clone());

    //     fm.create_note_file("Lie Group").unwrap();
    //     fm.create_note_file("Math/Atlas").unwrap();

    //     let tracker = crate::io::FileTracker::new(&config, tmp.clone()).unwrap();
    //     let builder = crate::io::HtmlBuilder::new(&config, tmp.clone());
    //     let index = crate::data::NoteIndex::new(tracker, builder).0;
    //     let index_con = std::rc::Rc::new(std::cell::RefCell::new(index));

    //     assert!(index_con.borrow().get("atlas").is_some());
    //     assert!(index_con.borrow().get("lie-group").is_some());

    //     fm.delete_note_file(&tmp.join(String::from("Lie Group.md")))
    //         .unwrap();
    //     fm.delete_note_file(
    //         &tmp.join(String::from("Math"))
    //             .join(String::from("Atlas.md")),
    //     )
    //     .unwrap();

    //     let (modifications, mut id_changes) = index_con.borrow_mut().handle_file_events().unwrap();
    //     id_changes.sort_unstable_by(|(a1, _b1), (a2, _b2)| a1.cmp(a2));

    //     assert!(modifications);
    //     assert_eq!(
    //         id_changes,
    //         vec![
    //             (String::from("atlas"), None),
    //             (String::from("lie-group"), None),
    //         ]
    //     );

    //     assert!(index_con.borrow().get("atlas").is_none());
    //     assert!(index_con.borrow().get("lie-group").is_none());
    // }
}
