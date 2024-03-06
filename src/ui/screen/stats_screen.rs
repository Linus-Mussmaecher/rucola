use crate::data::{Filter, Note, NoteStatistics};
use ratatui::{
    prelude::*,
    widgets::{self, *},
};
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

    pub fn filter(&mut self, filter: Filter) {
        self.stats = NoteStatistics::new_with_filters(&self.index, filter);
    }
}

impl super::Screen for StatsScreen {
    fn update(&mut self, msg: crate::ui::input::Message) -> Option<crate::ui::input::Message> {
        Some(msg)
    }

    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer) {
        // Vertical layout

        let vertical = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(17),
            Constraint::Length(17),
            Constraint::Min(5),
        ]);

        //  === General stats ===

        let general_stats_widths = [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Min(0),
        ];

        let [general_stats_area, bar_charts1_area, bar_charts2_area, filter_area] =
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
                    .take(8)
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
                    .take(8)
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
                    .take(8)
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
            .block(Block::bordered().title("Orphans".bold()))
            .bar_width(1)
            .bar_gap(1)
            .bar_style(Style::new().blue())
            .value_style(Style::new().dark_gray())
            .label_style(Style::new().cyan())
            .data(
                &self
                    .stats
                    .orphans
                    .iter()
                    .map(|name| (name.as_str(), 0))
                    .take(8)
                    .collect::<Vec<(&str, u64)>>(),
            )
            .max(10);

        // Filter area

        let filter = widgets::Paragraph::new("Here you will be able to filter notes, one day.")
            .block(Block::bordered().title("Filter".bold()));

        // === Rendering ===

        Widget::render(general_stats, general_stats_area, buf);
        Widget::render(bc11, bc11_area, buf);
        Widget::render(bc12, bc12_area, buf);
        Widget::render(bc21, bc21_area, buf);
        Widget::render(bc22, bc22_area, buf);
        Widget::render(filter, filter_area, buf);
    }
}
