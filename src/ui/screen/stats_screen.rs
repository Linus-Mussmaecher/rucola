use crate::data::{Filter, Note, NoteStatistics};
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
