use crate::data::{Filter, Note, NoteStatistics};
use ratatui::{prelude::*, widgets::*};
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
    pub fn new(index: Rc<HashMap<String, Note>>) -> Self {
        Self {
            stats: NoteStatistics::new_with_filters(&index, Filter::default()),
            index: index,
        }
    }

    pub fn filter(&mut self, filter: Filter) {}
}

impl super::Screen for StatsScreen{
    
    fn update(&mut self, msg: crate::ui::input::Message) -> Option<crate::ui::input::Message> {
        Some(msg)
    }

    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer) {

        
    }
}
