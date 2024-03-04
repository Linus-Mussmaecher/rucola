use super::note::Note;
use std::collections::HashMap;

/// A data struct containing statistical information about a (subset of a) user's notes.
#[derive(Debug, Clone)]
pub struct NoteStatistics {
    /// The total amount of words in the notes.
    /// What is a word and what not mirrors the definition from Note.words.
    word_count_total: usize,
    /// The total amount of characters, including whitespace, in the notes.
    char_count_total: usize,
    /// The total amount of notes tracked.
    note_count_total: usize,
    /// The total amount of _unique_ tags tracked.
    tag_count_total: usize,
    /// The total amount of (non-unique) links between notes. Does not count external links.
    link_count_total: usize,
    /// A vec of all tags used, along with the amount of notes under that tag. Sorted by descending usage by default.
    tag_usage: Vec<(String, usize)>,
    /// A vec of all note names, along with the total amount of links in other notes pointing to this note (or a heading in it.)
    inlinks: Vec<(String, usize)>,
    /// A vec of all note names, along with the amount of characters in the respective note.
    chars: Vec<(String, usize)>,
    /// A vec of all notes that have neither outgoing nor incoming links.
    orphans: Vec<String>,
}

impl NoteStatistics {
    /// Creates a new set of statistics from the subset of the passed index that matches the given filter.
    pub fn new_with_filters(index: &HashMap<String, Note>, filter: Filter) -> Self {
        // Filter the index
        let filtered_index = index
            .iter()
            .filter(|entry| {
                // Check if any or all the tags specified in the filter are in the note.
                let mut any_tag = false;
                let mut all_tags = true;
                for tag in filter.tags.iter() {
                    if entry.1.tags.contains(tag) {
                        any_tag = true;
                    } else {
                        all_tags = false;
                    }
                }

                // Check if any or all of the words specified in the filter are in the note title.
                let mut any_word = false;
                let mut all_words = true;
                for word in filter.title_words.iter() {
                    if entry.1.name.contains(word) {
                        any_word = true;
                    } else {
                        all_words = false;
                    }
                }

                // Compare with the filter settings
                (filter.all_tags && all_tags || any_tag)
                    && (filter.all_title_words && all_words || any_word)
            })
            .collect::<HashMap<&String, &Note>>();

        // Create a new hash map with tags and their usage
        let mut tags = HashMap::new();
        for (_, note) in filtered_index.iter() {
            // for every tag found in a note
            for tag in note.tags.iter() {
                // either create a new entry in the hash map or increment an existing entry by one
                match tags.get_mut(tag) {
                    Some(val) => *val += 1,
                    None => {
                        tags.insert(tag.clone(), 1 as usize);
                    }
                }
            }
        }

        // Create a new hash map with note names and the amount they are linked to from other notes.
        let mut inlinks = HashMap::new();
        for (_, note) in filtered_index.iter() {
            // for every link found within a note
            for link in note.links.iter() {
                // either create a new entry in the hash map or increment an existing entry by one.
                // Note that this does count self-links
                match inlinks.get_mut(link) {
                    Some(val) => *val += 1,
                    None => {
                        inlinks.insert(link.clone(), 1 as usize);
                    }
                }
            }
        }

        Self {
            // Sum up all word counts of notes
            word_count_total: filtered_index.values().map(|note| note.words).sum(),
            // Sum up all char counts of notes.
            char_count_total: filtered_index.values().map(|note| note.characters).sum(),
            // Simply take the lenght of the (filtered) index.
            note_count_total: filtered_index.len(),
            // Take the created tag-usage map and check its length.
            tag_count_total: tags.len(),
            // Take the sum of the length of links-lists from each individual note.
            link_count_total: filtered_index.values().map(|note| note.links.len()).sum(),
            // This is what the tag map was created for - just collect it into a vec and sort that.
            tag_usage: tags.into_iter().collect(),
            // Use filtered index and reduce the note to just the char count while cloning the name
            chars: filtered_index
                .iter()
                .map(|(&a, &b)| (a.clone(), b.characters))
                .collect(),
            // Use the filted index, take only those with no links and clone the name
            orphans: filtered_index
                .iter()
                .filter(|(&a, &b)| {
                    // needs to have no outgoing links and no incoming links i.e. no entry in the inlinks table
                    b.links.len() == 0 && inlinks.get(&a.to_lowercase().replace(" ", "-")).is_none()
                })
                .map(|(&a, _)| a.clone())
                .collect(),
            // This is what the inlinks map was created for - just collect and sort it.
            inlinks: inlinks.into_iter().collect(),
        }
    }
}

/// Describes a way to filter notes by their contained tags and/or title
#[derive(Debug, Default, Clone)]
pub struct Filter {
    /// Wether or not all specified tags must be contained in the note in order to match the filter, or only any (=at least one) of them.
    all_tags: bool,
    /// The tags to filter by, hash included.
    tags: Vec<String>,
    /// Wether or not all specified words must be contained in the note title in order to match the filter, or only any (=at least one) of them.
    all_title_words: bool,
    /// The words to search the note title for. No fuzzy matching, must fit completetely.
    title_words: Vec<String>,
}
