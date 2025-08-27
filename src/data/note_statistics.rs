use crate::{data, ui};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;

/// A struct describing statistics to a note in relation to a containing environment.
#[derive(Debug, Clone)]
pub struct NoteEnvStatistics {
    /// The notes id
    pub id: String,
    /// The fuzzy match score of this note with the filter used to create the environment
    match_score: i64,
    /// The amount of links pointing to this note from anywhere.
    inlinks_global: usize,
    /// The amount of links pointing to this note from other notes within the environment.
    inlinks_local: usize,
    /// The amount of links going out from this note to other notes within the environment.
    outlinks_local: usize,
    /// The amount of links going out from this note to other notes.
    /// Differs from the length of the 'links' table by not counting broken links.
    outlinks_global: usize,
    /// The amount of links originating from this note that do not have a valid target anywhere.
    broken_links: usize,
}

impl NoteEnvStatistics {
    /// Creates a new instance of NoteEnvStatistics with only the two passed fields filled out.
    fn new_empty(id: String, match_score: i64) -> Self {
        Self {
            id,
            match_score,
            inlinks_global: 0,
            inlinks_local: 0,
            outlinks_local: 0,
            outlinks_global: 0,
            broken_links: 0,
        }
    }

    /// Converts this note to a ratatui table row with its stats
    fn to_row(&self, index: data::NoteIndexContainer, styles: &ui::UiStyles) -> Option<Row<'_>> {
        // generate the stats row for each element
        index.borrow().get(&self.id).map(|note| {
            Row::new(vec![
                note.display_name.clone(),
                format!("{:7}", note.words),
                format!("{:7}", note.characters),
                format!("{:7}", self.outlinks_global),
                format!("{:7}", self.outlinks_local),
                format!("{:7}", self.inlinks_global),
                format!("{:7}", self.inlinks_local),
            ])
            .style(styles.text_style)
        })
    }
}

/// Describes the current sorting mode of the displayed list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum SortingMode {
    #[default]
    Name,
    Words,
    Chars,
    GlobalOutLinks,
    LocalOutLinks,
    GlobalInLinks,
    LocalInLinks,
    Score,
    Broken,
    LastModified,
}

/// A data struct containing statistical information about a (subset of a) user's notes.
/// This subset is called an 'environment' and is described by a filter passed to the constructor.
#[derive(Debug, Clone)]
pub struct EnvironmentStats {
    /// The total amount of words in the notes in this environment.
    /// What is a word and what not mirrors the definition from Note.words.
    word_count_total: usize,
    /// The total amount of characters, including whitespace, in the notes of this environment.
    char_count_total: usize,
    /// The total amount of notes in this environment.
    note_count_total: usize,
    /// The total amount of _unique_ tags in this environment.
    tag_count_total: usize,
    /// Total amount of links from a note within the environment to another note within the environment.
    local_local_links: usize,
    /// Total amount of links from a note within the environment to any note.
    local_global_links: usize,
    /// Total amount of links from any note to a note within the environment.
    global_local_links: usize,
    /// A vector of all notes within the environment.
    filtered_stats: Vec<NoteEnvStatistics>,
    /// Counts how many links among notes within the environment do not have a valid target anywhere.
    broken_links: usize,
}

impl EnvironmentStats {
    /// Creates a new set of statistics from the subset of the passed index that matches the given filter.
    pub fn new_with_filter(index: &super::NoteIndexContainer, filter: data::Filter) -> Self {
        let index = index.borrow();

        // Filter the index -> Create an iterator
        let mut filtered_index = index
            .inner
            .iter()
            .filter_map(|(id, note)| {
                filter.apply(note, &index).map(|score| {
                    (
                        id.clone(),
                        (NoteEnvStatistics::new_empty(id.clone(), score), note),
                    )
                })
            })
            .collect::<HashMap<_, _>>();

        // Count links by iterating over unfiltered index
        for (id, note) in index.inner.iter() {
            // Remember if source is from withing the environment.
            let local_source = filtered_index.contains_key(id);
            // Keep track of found local targets.
            let mut local_targets = 0;
            // Keep track of found targets.
            let mut global_targets = 0;

            // Then go over its links.
            for link in &note.links {
                // Check if target exists
                if index.inner.contains_key(link) {
                    // and increase count of valid targets if so.
                    global_targets += 1;

                    // Now check if target is local.
                    if let Some((target, _)) = filtered_index.get_mut(link) {
                        // Always count up global inlink count of target.
                        target.inlinks_global += 1;
                        // If id of source is also in filtered index, also count up local inlink count of target.
                        if local_source {
                            target.inlinks_local += 1;
                        }
                        // Since this target was in the environment, increment the counter.
                        local_targets += 1;
                    }
                }
            }
            // If source was local, we are interested in its stats.
            // Add the found local/global targets to the statistics (this could be an assignment).
            if let Some((source, _)) = filtered_index.get_mut(id) {
                source.outlinks_local += local_targets;
                source.outlinks_global += global_targets;
                source.broken_links = note.links.len() - global_targets;
            }
        }

        Self {
            // Word count: Just map over the stats.
            word_count_total: filtered_index.values().map(|(_, stats)| stats.words).sum(),
            // Char count: Just map over the stats
            char_count_total: filtered_index
                .values()
                .map(|(_, stats)| stats.characters)
                .sum(),
            // Total notes: Just the length of the filtered index.
            note_count_total: filtered_index.len(),
            // Total tags: Collect all tag vectors of notes into a HashSet, then take its length.
            tag_count_total: filtered_index
                .values()
                .flat_map(|(_, stats)| &stats.tags)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            // Local-Local links: Check outgoing local links of all notes. Could also check incoming local links of all notes.
            local_local_links: filtered_index
                .values()
                .map(|(env_stats, _)| env_stats.outlinks_local)
                .sum(),
            // Local-Global links: Check outgoing global links of all notes.
            local_global_links: filtered_index
                .values()
                .map(|(env_stats, _)| env_stats.outlinks_global)
                .sum(),
            // Global-Local Links: Check incoming links of all notes.
            global_local_links: filtered_index
                .values()
                .map(|(env_stats, _)| env_stats.inlinks_global)
                .sum(),
            // Broken links: Again, just sum it up.
            broken_links: filtered_index
                .values()
                .map(|(env_stats, _)| env_stats.broken_links)
                .sum(),
            // Finally, reduce the vector to just the env stats
            filtered_stats: {
                let mut fs = filtered_index
                    .into_values()
                    .map(|(env_stats, _)| env_stats)
                    .collect::<Vec<_>>();

                // Default sort: By match score, descending.
                fs.sort_by_cached_key(|env_stats| env_stats.match_score);
                fs.reverse();

                fs
            },
        }
    }

    /// Returns the nth element of the underlying sorted vector
    pub fn get_selected(&self, index: usize) -> Option<&NoteEnvStatistics> {
        self.filtered_stats.get(index)
    }

    /// Sorts the underlying vec
    pub fn sort(&mut self, index: data::NoteIndexContainer, mode: SortingMode, ascending: bool) {
        // Always sort by name first
        self.filtered_stats
            .sort_by_cached_key(|env_stats| env_stats.id.clone());

        // If the sorting mode is not name, now sort by the actual sorting mode.
        if mode != SortingMode::Name {
            // If sorting in reverse is desired, pre-reverse this, so when reversing again later, the list will still be sub-sorted by name ascendingly.
            if !ascending {
                self.filtered_stats.reverse();
            }
            // all others are usize and can be done in one thing
            self.filtered_stats.sort_by_cached_key(|env_stats| {
                if let Some(note) = index.borrow().get(&env_stats.id) {
                    match mode {
                        // This should not appear
                        SortingMode::Name => 0,
                        // These should appear
                        SortingMode::Words => note.words,
                        SortingMode::Chars => note.characters,
                        SortingMode::GlobalOutLinks => env_stats.outlinks_global,
                        SortingMode::LocalOutLinks => env_stats.outlinks_local,
                        SortingMode::GlobalInLinks => env_stats.inlinks_global,
                        SortingMode::LocalInLinks => env_stats.inlinks_local,
                        SortingMode::Score => env_stats.match_score as usize,
                        SortingMode::Broken => env_stats.broken_links,
                        SortingMode::LastModified => {
                            // Get the file's modification time as seconds since epoch
                            note.last_modification
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs() as usize)
                                // If no time since modification can be determined, use a value that will put this note at the end of the sorted list.
                                .unwrap_or(if ascending { usize::MAX } else { usize::MIN })
                        }
                    }
                } else {
                    0
                }
            })
        }

        // Potentially reverse sorting
        if !ascending {
            self.filtered_stats.reverse();
        }
    }

    /// Returns the amount of notes in this environment.
    pub fn len(&self) -> usize {
        self.filtered_stats.len()
    }

    /// Converts this environemnt to a table of rows with the (sorted) notes contained in it.
    pub fn to_note_table(
        &self,
        index: data::NoteIndexContainer,
        styles: &ui::UiStyles,
    ) -> Table<'_> {
        // Calculate widths
        let notes_table_widths = [
            Constraint::Min(25),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        // Construct rows
        let notes_rows = self
            .filtered_stats
            .iter()
            .flat_map(|note_env| note_env.to_row(index.clone(), styles))
            .collect::<Vec<Row>>();

        Table::new(notes_rows, notes_table_widths).column_spacing(1)
    }

    /// Converts this environment statistics struct to a ratatui table with the basic, global stats.
    pub fn to_global_stats_table(&self, styles: &ui::UiStyles) -> Table<'_> {
        // Horizontal layout
        let stats_widths = [
            Constraint::Length(20),
            Constraint::Length(16),
            Constraint::Length(20),
            Constraint::Length(16),
            Constraint::Min(0),
        ];

        //  === Global stats ===

        let global_stats_rows = [
            Row::new(vec![
                Cell::from("Total notes:").style(styles.text_style),
                Cell::from(format!("{:7}", self.note_count_total)).style(styles.text_style),
                Cell::from("Total words:").style(styles.text_style),
                Cell::from(format!("{:7}", self.word_count_total)).style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("Total unique tags:").style(styles.text_style),
                Cell::from(format!("{:7}", self.tag_count_total)).style(styles.text_style),
                Cell::from("Total characters:").style(styles.text_style),
                Cell::from(format!("{:7}", self.char_count_total)).style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("Total links:").style(styles.text_style),
                Cell::from(format!("{:7}", self.local_local_links)).style(styles.text_style),
                Cell::from("Broken links:").style(styles.text_style),
                Cell::from(format!("{:7}", self.broken_links)).style(styles.text_style),
            ]),
        ];

        Table::new(global_stats_rows, stats_widths).column_spacing(1)
    }

    /// Converts this environment statistics struct to a ratatui table with the full, local stats.
    pub fn to_local_stats_table(&self, global: &Self, styles: &ui::UiStyles) -> Table<'_> {
        // Horizontal layout
        let stats_widths = [
            Constraint::Length(20),
            Constraint::Length(16),
            Constraint::Length(20),
            Constraint::Length(16),
            Constraint::Min(0),
        ];

        //  === Local stats ===
        let local_stats_rows = [
            Row::new(vec![
                Cell::from("Total notes:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.note_count_total,
                    self.note_count_total * 100 / global.note_count_total.max(1)
                ))
                .style(styles.text_style),
                Cell::from("Total words:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.word_count_total,
                    self.word_count_total * 100 / global.word_count_total.max(1)
                ))
                .style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("Total unique tags:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.tag_count_total,
                    self.tag_count_total * 100 / global.tag_count_total.max(1)
                ))
                .style(styles.text_style),
                Cell::from("Total characters:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.char_count_total,
                    self.char_count_total * 100 / global.char_count_total.max(1)
                ))
                .style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("Incoming links:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.global_local_links,
                    self.global_local_links * 100 / global.local_local_links.max(1),
                ))
                .style(styles.text_style),
                Cell::from("Outgoing links:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_global_links,
                    self.local_global_links * 100 / global.local_local_links.max(1),
                ))
                .style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("Internal links:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_local_links,
                    self.local_local_links * 100 / global.local_local_links.max(1),
                ))
                .style(styles.text_style),
                Cell::from("Broken links:").style(styles.text_style),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.broken_links,
                    self.broken_links * 100 / global.broken_links.max(1)
                ))
                .style(styles.text_style),
            ]),
        ];

        Table::new(local_stats_rows, stats_widths).column_spacing(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data, io};

    #[test]
    fn test_env_stats_1_tags_any() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        let index = std::rc::Rc::new(std::cell::RefCell::new(index));

        // === Filter 1 ===

        let filter1 = data::Filter {
            any: true,
            tags: vec![
                ("#topology".to_string(), true),
                ("#diffgeo".to_string(), true),
            ],
            links: vec![],
            blinks: vec![],
            title: String::new(),
            full_text: None,
        };

        let env1 = EnvironmentStats::new_with_filter(&index, filter1);

        assert_eq!(env1.note_count_total, 5);
        assert_eq!(env1.tag_count_total, 3);
        assert_eq!(env1.local_local_links, 10);
        assert_eq!(env1.local_global_links, 11);
        assert_eq!(env1.global_local_links, 13);
        assert_eq!(env1.broken_links, 1);
    }

    #[test]
    fn test_env_stats_2_tags_all() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        let index = std::rc::Rc::new(std::cell::RefCell::new(index));
        // === Filter 2 ===

        let filter2 = data::Filter {
            any: false,
            tags: vec![
                ("#topology".to_string(), true),
                ("#diffgeo".to_string(), true),
            ],
            links: vec![],
            blinks: vec![],
            title: String::new(),
            full_text: None,
        };
        let env2 = EnvironmentStats::new_with_filter(&index, filter2);

        assert_eq!(env2.note_count_total, 2);
        assert_eq!(env2.tag_count_total, 2);
        assert_eq!(env2.local_local_links, 1);
        assert_eq!(env2.local_global_links, 5);
        assert_eq!(env2.global_local_links, 6);
        assert_eq!(env2.broken_links, 1);

        env2.filtered_stats
            .iter()
            .filter(|env_stats| env_stats.id == "manifold")
            .for_each(|ma| {
                assert_eq!(ma.inlinks_global, 5);
                assert_eq!(ma.inlinks_local, 1);
                assert_eq!(ma.outlinks_local, 0);
                assert_eq!(ma.outlinks_global, 4);
                assert_eq!(ma.broken_links, 0);
            });
    }

    #[test]
    fn test_env_stats_3_title() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        let index = std::rc::Rc::new(std::cell::RefCell::new(index));

        // === Filter 3 ===

        let filter3 = data::Filter {
            any: false,
            tags: vec![],
            links: vec![],
            blinks: vec![],
            title: "operating".to_string(),
            full_text: None,
        };
        let env3 = EnvironmentStats::new_with_filter(&index, filter3);

        assert_eq!(env3.note_count_total, 1);
        assert_eq!(env3.tag_count_total, 1);
        assert_eq!(env3.local_local_links, 0);
        assert_eq!(env3.local_global_links, 6);
        assert_eq!(env3.global_local_links, 0);
        assert_eq!(env3.broken_links, 0);
    }

    #[test]
    fn test_env_stats_4_blinks() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        let index = std::rc::Rc::new(std::cell::RefCell::new(index));

        // === Filter 4 ===

        let filter4 = data::Filter {
            any: true,
            tags: vec![],
            links: vec![],
            blinks: vec![("atlas".to_string(), true)],
            title: String::new(),
            full_text: None,
        };
        let env4 = EnvironmentStats::new_with_filter(&index, filter4);

        assert_eq!(env4.note_count_total, 3);
        assert_eq!(env4.tag_count_total, 2);
        assert_eq!(env4.local_local_links, 2);
        assert_eq!(env4.local_global_links, 5);
        assert_eq!(env4.global_local_links, 9);
        assert_eq!(env4.broken_links, 1);
    }

    #[test]
    fn test_env_stats_5_links_blinks() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        let index = std::rc::Rc::new(std::cell::RefCell::new(index));

        // === Filter 5 ===

        let filter5 = data::Filter {
            any: true,
            tags: vec![],
            links: vec![("smooth-map".to_string(), true)],
            blinks: vec![("atlas".to_string(), true)],
            title: String::new(),
            full_text: None,
        };
        let env5 = EnvironmentStats::new_with_filter(&index, filter5);

        assert_eq!(env5.note_count_total, 4);
        assert_eq!(env5.tag_count_total, 3);
        assert_eq!(env5.local_local_links, 5);
        assert_eq!(env5.local_global_links, 8);
        assert_eq!(env5.global_local_links, 10);
        assert_eq!(env5.broken_links, 1);
    }
}
