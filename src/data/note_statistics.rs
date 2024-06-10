use crate::data;
use std::collections::HashMap;

/// A struct describing statistics to a note in relation to a containing environment.
#[derive(Debug, Clone)]
pub struct NoteEnvStatistics {
    /// The notes id
    pub id: String,
    /// The fuzzy match score of this note with the filter used to create the environment
    pub match_score: i64,
    /// The amount of links pointing to this note from anywhere.
    pub inlinks_global: usize,
    /// The amount of links pointing to this note from other notes within the environment.
    pub inlinks_local: usize,
    /// The amount of links going out from this note to other notes within the environment.
    pub outlinks_local: usize,
    /// The amount of links going out from this note to other notes.
    /// Differs from the length of the 'links' table by not counting broken links.
    pub outlinks_global: usize,
    /// The amount of links originating from this note that do not have a valid target anywhere.
    pub broken_links: usize,
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
}

/// A data struct containing statistical information about a (subset of a) user's notes.
/// This subset is called an 'environment' and is described by a filter passed to the constructor.
#[derive(Debug, Clone)]
pub struct EnvironmentStats {
    /// The total amount of words in the notes in this environment.
    /// What is a word and what not mirrors the definition from Note.words.
    pub word_count_total: usize,
    /// The total amount of characters, including whitespace, in the notes of this environment.
    pub char_count_total: usize,
    /// The total amount of notes in this environment.
    pub note_count_total: usize,
    /// The total amount of _unique_ tags in this environment.
    pub tag_count_total: usize,
    /// Total amount of links from a note within the environment to another note within the environment.
    pub local_local_links: usize,
    /// Total amount of links from a note within the environment to any note.
    pub local_global_links: usize,
    /// Total amount of links from any note to a note within the environment.
    pub global_local_links: usize,
    /// A vector of all notes within the environment.
    pub filtered_stats: Vec<NoteEnvStatistics>,
    /// Counts how many links among notes within the environment do not have a valid target anywhere.
    pub broken_links: usize,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data, files};

    #[test]
    fn test_env_stats_general() {
        let config = files::Config::default();
        let tracker = files::FileTracker::new(&config);
        let builder = files::HtmlBuilder::new(&config);
        let index = data::NoteIndex::new(tracker, builder);

        assert_eq!(index.inner.len(), 11);

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
        };

        let env1 = EnvironmentStats::new_with_filter(&index, filter1);

        assert_eq!(env1.note_count_total, 5);
        assert_eq!(env1.tag_count_total, 3);
        assert_eq!(env1.local_local_links, 9);
        assert_eq!(env1.local_global_links, 10);
        assert_eq!(env1.global_local_links, 12);
        assert_eq!(env1.broken_links, 1);

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
        };
        let env2 = EnvironmentStats::new_with_filter(&index, filter2);

        assert_eq!(env2.note_count_total, 2);
        assert_eq!(env2.tag_count_total, 2);
        assert_eq!(env2.local_local_links, 1);
        assert_eq!(env2.local_global_links, 5);
        assert_eq!(env2.global_local_links, 5);
        assert_eq!(env2.broken_links, 1);

        env2.filtered_stats
            .iter()
            .filter(|env_stats| env_stats.id == "manifold")
            .for_each(|ma| {
                assert_eq!(ma.inlinks_global, 4);
                assert_eq!(ma.inlinks_local, 1);
                assert_eq!(ma.outlinks_local, 0);
                assert_eq!(ma.outlinks_global, 4);
                assert_eq!(ma.broken_links, 0);
            });

        // === Filter 3 ===

        let filter3 = data::Filter {
            any: false,
            tags: vec![],
            links: vec![],
            blinks: vec![],
            title: "operating".to_string(),
        };
        let env3 = EnvironmentStats::new_with_filter(&index, filter3);

        assert_eq!(env3.note_count_total, 1);
        assert_eq!(env3.tag_count_total, 1);
        assert_eq!(env3.local_local_links, 0);
        assert_eq!(env3.local_global_links, 6);
        assert_eq!(env3.global_local_links, 0);
        assert_eq!(env3.broken_links, 0);

        // === Filter 4 ===

        let filter4 = data::Filter {
            any: true,
            tags: vec![],
            links: vec![],
            blinks: vec![("atlas".to_string(), true)],
            title: String::new(),
        };
        let env4 = EnvironmentStats::new_with_filter(&index, filter4);

        assert_eq!(env4.note_count_total, 3);
        assert_eq!(env4.tag_count_total, 2);
        assert_eq!(env4.local_local_links, 2);
        assert_eq!(env4.local_global_links, 5);
        assert_eq!(env4.global_local_links, 8);
        assert_eq!(env4.broken_links, 1);

        // === Filter 5 ===

        let filter5 = data::Filter {
            any: true,
            tags: vec![],
            links: vec![("smooth-map".to_string(), true)],
            blinks: vec![("atlas".to_string(), true)],
            title: String::new(),
        };
        let env5 = EnvironmentStats::new_with_filter(&index, filter5);

        assert_eq!(env5.note_count_total, 4);
        assert_eq!(env5.tag_count_total, 3);
        assert_eq!(env5.local_local_links, 5);
        assert_eq!(env5.local_global_links, 8);
        assert_eq!(env5.global_local_links, 9);
        assert_eq!(env5.broken_links, 1);
    }
}
