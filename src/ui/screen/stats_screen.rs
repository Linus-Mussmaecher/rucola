use crate::data::{Filter, Note, NoteStatistics};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{self, *},
};
use std::{collections::HashMap, rc::Rc};
use tui_textarea::TextArea;

#[derive(Clone, Debug, PartialEq, Eq)]
enum StatsMode {
    Main,
    Filter,
}

/// The Stats screen shows the user statistical information about their notes
#[derive(Clone)]
pub struct StatsScreen {
    /// A reference to the index of all notes
    index: Rc<HashMap<String, Note>>,
    /// The currently displayed statistics
    stats: NoteStatistics,
    /// The text area to type in filters.
    text_area: TextArea<'static>,
    /// Current input mode
    mode: StatsMode,
}

impl StatsScreen {
    /// Creates a new stats screen, with no filter applied by default
    pub fn new(index: Rc<HashMap<String, Note>>) -> Self {
        Self {
            stats: NoteStatistics::new_with_filters(&index, Filter::default()),
            index,
            text_area: {
                let mut a = TextArea::default();
                a.set_block(Block::bordered().title("Filter".bold()));
                a
            },
            mode: StatsMode::Main,
        }
    }

    /// Check the current values of the text area, create filter from it and apply it
    fn filter_by_area(&mut self) {
        // We should only have one line, read that one
        if let Some(line) = self.text_area.lines().first() {
            let mut filter = Filter::default();

            // Go through words
            for word in line.split_whitespace() {
                if word.starts_with('#') {
                    // words with a hash count as a tag
                    filter.tags.push(word.to_string());
                } else {
                    // other words are searched for in the title
                    filter.title_words.push(word.to_string());
                }
            }
            // apply filter to displayed statistic
            self.filter(filter);
        }
    }

    /// Reloads the displayed statistics, showing stats for only those elements of the index matching the specified filter.
    fn filter(&mut self, filter: Filter) {
        self.stats = NoteStatistics::new_with_filters(&self.index, filter);
    }
}

impl super::Screen for StatsScreen {
    fn update(&mut self, key: KeyEvent) -> Option<crate::ui::input::Message> {
        // Check for mode
        match self.mode {
            // Main mode: Switch to modes, general command
            StatsMode::Main => match key.code {
                // F: Go to filter mode
                KeyCode::Char('f') => {
                    self.mode = StatsMode::Filter;
                }
                // C: Clear filter
                KeyCode::Char('c') => {
                    self.text_area.select_all();
                    self.text_area.cut();
                    self.filter(Filter::default());
                }
                // Q: Quit application
                KeyCode::Char('q') => return Some(crate::ui::input::Message::Quit),
                // S: Select screen
                KeyCode::Char('s') => return Some(crate::ui::input::Message::SwitchSelect),
                // R: Reload
                KeyCode::Char('r') => return Some(crate::ui::input::Message::SwitchStats),
                _ => {}
            },
            // Filter mode: Type in filter values
            StatsMode::Filter => {
                match key.code {
                    // Escape: Back to main mode
                    KeyCode::Esc => {
                        self.mode = StatsMode::Main;
                    }
                    // Enter: Apply filter, back to main mode
                    KeyCode::Enter => {
                        self.filter_by_area();
                        self.mode = StatsMode::Main;
                    }
                    // All other key events are passed on to the text area
                    _ => {
                        // Else -> Pass on to the text area
                        self.text_area.input_without_shortcuts(key);
                    }
                };
            }
        }

        None
    }

    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer) {
        // Vertical layout

        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(17),
            Constraint::Min(17),
        ]);

        //  === General stats ===

        let general_stats_widths = [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Min(0),
        ];

        let [filter_area, general_stats_area, bar_charts1_area, bar_charts2_area] =
            vertical.areas(area);

        let strings = [
            self.stats.note_count_total.to_string(),
            self.stats.word_count_total.to_string(),
            self.stats.tag_count_total.to_string(),
            self.stats.char_count_total.to_string(),
            self.stats.link_count_total.to_string(),
        ];

        let general_stats_rows = [
            Row::new(vec![
                "Total notes:",
                &strings[0],
                "Total words: ",
                &strings[1],
            ]),
            Row::new(vec![
                "Total unique tags:",
                &strings[2],
                "Total characters: ",
                &strings[3],
            ]),
            Row::new(vec!["Total links:", &strings[4], "", ""]),
        ];

        let general_stats = widgets::Table::new(general_stats_rows, general_stats_widths)
            .column_spacing(1)
            .block(Block::bordered().title("Total Statistics".bold()));

        // === First two bar chars ===

        let [bc11_area, bc12_area] =
            Layout::horizontal(Constraint::from_percentages([50, 50])).areas(bar_charts1_area);

        let bc11 = BarChart::default()
            .direction(Direction::Horizontal)
            .block(Block::bordered().title("Most used tags".bold()))
            .bar_width(1)
            .bar_gap(1)
            .bar_style(Style::new().blue())
            .value_style(Style::new().dark_gray())
            .label_style(Style::new().cyan())
            .data(
                &self
                    .stats
                    .tag_usage
                    .iter()
                    .map(|(name, value)| (name.as_str(), *value as u64))
                    .collect::<Vec<(&str, u64)>>(),
            )
            .max(
                self.stats
                    .tag_usage
                    .first()
                    .map(|(_, val)| *val as u64)
                    .unwrap_or(10),
            );

        let bc12 = BarChart::default()
            .direction(Direction::Horizontal)
            .block(Block::bordered().title("Longest notes (words)".bold()))
            .bar_width(1)
            .bar_gap(1)
            .bar_style(Style::new().blue())
            .value_style(Style::new().dark_gray())
            .label_style(Style::new().cyan())
            .data(
                &self
                    .stats
                    .words
                    .iter()
                    .map(|(name, value)| (name.as_str(), *value as u64))
                    .collect::<Vec<(&str, u64)>>(),
            )
            .max(
                self.stats
                    .words
                    .first()
                    .map(|(_, val)| *val as u64 + 10)
                    .unwrap_or(10),
            );

        // === Second two bar chars ===

        let [bc21_area, bc22_area] =
            Layout::horizontal(Constraint::from_percentages([50, 50])).areas(bar_charts2_area);

        let bc21 = BarChart::default()
            .direction(Direction::Horizontal)
            .block(Block::bordered().title("Most linked files".bold()))
            .bar_width(1)
            .bar_gap(1)
            .bar_style(Style::new().blue())
            .value_style(Style::new().dark_gray())
            .label_style(Style::new().cyan())
            .data(
                &self
                    .stats
                    .inlinks
                    .iter()
                    .map(|(name, value)| (name.as_str(), *value as u64))
                    .collect::<Vec<(&str, u64)>>(),
            )
            .max(
                self.stats
                    .inlinks
                    .first()
                    .map(|(_, val)| *val as u64 + 10)
                    .unwrap_or(10),
            );

        let bc22 = BarChart::default()
            .direction(Direction::Horizontal)
            .block(Block::bordered().title("Most linked files (global)".bold()))
            .bar_width(1)
            .bar_gap(1)
            .bar_style(Style::new().blue())
            .value_style(Style::new().dark_gray())
            .label_style(Style::new().cyan())
            .data(
                &self
                    .stats
                    .inlinks_global
                    .iter()
                    .map(|(name, value)| (name.as_str(), *value as u64))
                    .collect::<Vec<(&str, u64)>>(),
            )
            .max(
                self.stats
                    .inlinks_global
                    .first()
                    .map(|(_, val)| *val as u64 + 10)
                    .unwrap_or(10),
            );

        // Filter area
        let filter_input = self.text_area.widget();

        // === Rendering ===

        Widget::render(filter_input, filter_area, buf);

        Widget::render(general_stats, general_stats_area, buf);
        Widget::render(bc11, bc11_area, buf);
        Widget::render(bc12, bc12_area, buf);
        Widget::render(bc21, bc21_area, buf);
        Widget::render(bc22, bc22_area, buf);
    }
}
