use std::path;
use std::sync::mpsc::{self, TryIter};

use notify::Watcher;

/// Stores configuration to track the file system the notes are stored in.
#[derive(Debug)]
pub struct FileTracker {
    /// Path to the vault to index.
    vault_path: path::PathBuf,
    /// File types to consider notes
    file_types: ignore::types::Types,
    /// The main gitignore files in the vault
    gitignore: Option<ignore::gitignore::Gitignore>,
    /// Watcher that checks for file changes in the vault directory and needs to be kept alive with this index.
    /// Can be unused because it is just here for RAII.
    #[allow(unused)]
    watcher: notify::INotifyWatcher,
    /// Channel from which file change events in the vault directory are deposited by the watcher and can be requested.
    file_change_channel: mpsc::Receiver<Result<notify::Event, notify::Error>>,
}

impl FileTracker {
    pub fn new(config: &super::config::Config) -> Self {
        // Pre-calculate allowed file types
        let mut types_builder = ignore::types::TypesBuilder::new();
        types_builder.add_defaults();
        for name in config.file_types.iter() {
            types_builder.select(name);
        }

        // Get vault path
        let vault_path = config
            .vault_path
            .clone()
            .unwrap_or_else(|| std::env::current_dir().expect("To get current working directory."));
        // Search and fetch gitignore
        let gitignore_builder = ignore::gitignore::GitignoreBuilder::new(&vault_path);
        // Create asynchronous channel for file events.
        let (sender, receiver) = mpsc::channel();

        // Create watcher so we can store it in the file, delaying its drop (which stops its function) until the end of the lifetime of this index.
        let mut watcher = notify::recommended_watcher(move |res| {
            sender.send(res).unwrap();
        })
        .unwrap();

        // Start watching the vault.
        watcher
            .watch(&vault_path, notify::RecursiveMode::Recursive)
            .expect("Fixed config does not fail.");

        Self {
            vault_path,
            file_types: types_builder
                .build()
                .expect("To build predefined types correctly."),
            gitignore: gitignore_builder.build().ok(),
            watcher,
            file_change_channel: receiver,
        }
    }
    /// Returns a file walker that iterates over all notes to index.
    pub fn get_walker(&self) -> ignore::Walk {
        ignore::WalkBuilder::new(&self.vault_path)
            .types(self.file_types.clone())
            .build()
    }

    /// Wether the given path is supposed to be tracked by rucola or not.
    /// Checks for file endings and gitignore
    pub fn is_tracked(&self, path: &path::PathBuf) -> bool {
        let file_ending = if let ignore::Match::Whitelist(_) = self.file_types.matched(path, false)
        {
            true
        } else {
            false
        };

        let gitignore = self
            .gitignore
            .as_ref()
            .map(|gi| {
                if let ignore::Match::Ignore(_) = gi.matched(path, false) {
                    false
                } else {
                    true
                }
            })
            .unwrap_or(true);

        return file_ending && gitignore;
    }

    /// Returns an iterator over all events found by this tracker since the last check.
    pub fn try_events_iter(&self) -> TryIter<'_, Result<notify::Event, notify::Error>> {
        self.file_change_channel.try_iter()
    }
}
#[cfg(test)]
mod tests {
    use crate::files;

    #[test]
    fn test_file_endings() {
        let no_ending_tar = std::path::PathBuf::from("./tests/common/test");
        let md_ending_tar = std::path::PathBuf::from("./tests/common/test.md");
        let txt_ending_tar = std::path::PathBuf::from("./tests/common/test.txt");
        let tex_ending_tar = std::path::PathBuf::from("./tests/common/test.tex");

        let config = files::Config::default();
        let tracker = super::FileTracker::new(&config);

        assert!(!tracker.is_tracked(&no_ending_tar));
        assert!(tracker.is_tracked(&md_ending_tar));
        assert!(!tracker.is_tracked(&txt_ending_tar));
        assert!(!tracker.is_tracked(&tex_ending_tar));

        let tracker = super::FileTracker::new(&files::config::Config {
            file_types: vec!["md".to_owned(), "txt".to_owned()],
            ..Default::default()
        });

        assert!(!tracker.is_tracked(&no_ending_tar));
        assert!(tracker.is_tracked(&md_ending_tar));
        assert!(tracker.is_tracked(&txt_ending_tar));
        assert!(!tracker.is_tracked(&tex_ending_tar));

        let tracker = super::FileTracker::new(&files::config::Config {
            file_types: vec!["all".to_owned()],
            ..Default::default()
        });

        assert!(!tracker.is_tracked(&no_ending_tar));
        assert!(tracker.is_tracked(&md_ending_tar));
        assert!(tracker.is_tracked(&txt_ending_tar));
        assert!(tracker.is_tracked(&tex_ending_tar));
    }
}
