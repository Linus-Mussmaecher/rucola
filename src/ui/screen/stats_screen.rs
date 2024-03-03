use crate::data::Note;
use std::{collections::HashMap, rc::Rc};

/// The Stats screen shows the user statistical information about their notes
#[derive(Clone, Debug)]
pub struct StatsScreen {
    /// A reference to the index of all notes
    index: Rc<HashMap<String, Note>>,
    /// The currently displayed statistics
    stats: NoteStatistics,
}

impl StatsScreen {
    /// Creates a new stats screen, with no filter applied by default
    pub fn new(index: &Rc<HashMap<String, Note>>) -> Self {
        Self {
            index: index.clone(),
            stats: NoteStatistics::new_with_filters(&index, Filter::default()),
        }
    }

    pub fn filter(&mut self, filter: Filter) {}
}

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
    pub fn new_with_filters(index: &HashMap<String, Note>, filter: Filter) -> Self {
        let filtered_index = index
            .iter()
            .filter(|entry| {
                let mut any_tag = false;
                let mut all_tags = true;
                for tag in filter.tags.iter() {
                    if entry.1.tags.contains(tag) {
                        any_tag = true;
                    } else {
                        all_tags = false;
                    }
                }

                let mut any_word = false;
                let mut all_words = true;

                for word in filter.title_words.iter() {
                    if entry.1.name.contains(word) {
                        any_word = true;
                    } else {
                        all_words = false;
                    }
                }

                (filter.all_tags && all_tags || any_tag)
                    && (filter.all_title_words && all_words || any_word)
            })
            .collect::<HashMap<&String, &Note>>();

        let mut tags = HashMap::new();

        for (_, note) in filtered_index.iter() {
            for tag in note.tags.iter() {
                match tags.get_mut(tag) {
                    Some(val) => *val += 1,
                    None => {
                        tags.insert(tag, 1 as usize);
                    }
                }
            }
        }

        Self {
            word_count_total: filtered_index.values().map(|note| note.words).sum(),
            char_count_total: filtered_index.values().map(|note| note.characters).sum(),
            note_count_total: filtered_index.len(),
            tag_count_total: tags.len(),
            link_count_total: filtered_index.values().map(|note| note.links.len()).sum(),
            tag_usage: tags.into_iter().map(|(a, b)| (a.clone(), b)).collect(),
            inlinks: todo!(),
            chars: todo!(),
            orphans: todo!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Filter {
    all_tags: bool,
    tags: Vec<String>,
    all_title_words: bool,
    title_words: Vec<String>,
}
