use crate::{
    data::{Filter, Note, NoteStatistics},
    ui::Styles,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{self, *},
};

use std::{collections::HashMap, rc::Rc};
use tui_textarea::TextArea;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SelectMode {
    #[default]
    Select,
    Filter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SortingMode {
    #[default]
    Name,
    Words,
    Chars,
    OutLinks,
    GlobalInLinks,
    LocalInLinks,
}

/// The select screen shows the user statistical information about their notes and allows them to select one for display.
pub struct SelectScreen {
    // === Displayed Data ===
    /// A reference to the index of all notes
    index: Rc<HashMap<String, Note>>,
    /// The currently displayed statistics for all notes.
    local_stats: NoteStatistics,
    /// The currently displayed statistics for all notes matching the current filter.
    global_stats: NoteStatistics,
    /// The styles used on this screen.
    styles: Styles,

    // === UI (state) ===
    /// The text area to type in filters.
    text_area: TextArea<'static>,
    /// Current input mode
    mode: SelectMode,
    /// Current state of the list
    ///
    /// This is saved as a simple usize from which the ListState to use with ratatui is constructed in immediate mode.
    /// This allows us to convert only the neccessary notes to ListItems and save some time.
    selected: usize,

    // === UI options ===
    /// UI mode wether the user wants to filter for all tags or any tags.
    all_tags: bool,
    /// Ui mode for the chosen sorting variant
    sorting: SortingMode,
    /// Reversed sorting
    sorting_rev: bool,
}

impl SelectScreen {
    /// Creates a new stats screen, with no filter applied by default
    pub fn new(index: Rc<HashMap<String, Note>>) -> Self {
        let styles = Styles::default();
        let mut res = Self {
            local_stats: NoteStatistics::new_with_filters(&index, Filter::default()),
            global_stats: NoteStatistics::new_with_filters(&index, Filter::default()),
            index,
            text_area: TextArea::default(),
            mode: SelectMode::Select,
            styles,
            all_tags: false,
            sorting: SortingMode::Name,
            sorting_rev: false,
            selected: 0,
        };

        res.style_text_area();

        res
    }

    /// Styling of TextArea extracted from constructor to keep it clean.
    fn style_text_area(&mut self) {
        // The actual title
        let title_top = block::Title::from(Line::from(vec![
            Span::styled("F", self.styles.hotkey_style),
            Span::styled("ilter", self.styles.title_style),
        ]));

        // The hotkey instructions at the bottom.
        let instructions = block::Title::from(Line::from(vec![
            Span::styled("C", self.styles.hotkey_style),
            Span::styled("lear filter", self.styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Top);

        let instructions_bot = block::Title::from(Line::from(vec![
            Span::styled(
                if self.all_tags { "All " } else { "Any " },
                self.styles.text_style,
            ),
            Span::styled("T", self.styles.hotkey_style),
            Span::styled("ags", self.styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Bottom);

        self.text_area.set_style(self.styles.input_style);
        self.text_area
            .set_cursor_line_style(self.styles.input_style);

        self.text_area.set_block(
            Block::bordered()
                .title(title_top)
                .title(instructions)
                .title(instructions_bot),
        );
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
            // check for any or all tags
            filter.all_tags = self.all_tags;

            // apply filter to displayed statistic
            self.filter(filter);
        }
    }

    /// Reloads the displayed statistics, showing stats for only those elements of the index matching the specified filter.
    fn filter(&mut self, filter: Filter) {
        self.local_stats = NoteStatistics::new_with_filters(&self.index, filter);
        self.sort(None);
    }

    /// Sets a new sorting mode if requested.
    /// Then sorts the note display according to the current sorting mode.
    fn sort(&mut self, new_mode: Option<SortingMode>) {
        // if a new sorting mode was requested
        if let Some(new_mode) = new_mode {
            if new_mode == self.sorting {
                // if the new sorting mode is also the old one, reverse the sorting
                self.sorting_rev = !self.sorting_rev;
            } else {
                // else, apply the new mode but leave the reversed sorting as is
                self.sorting = new_mode;
            }
        }
        if self.sorting == SortingMode::Name {
            // Name: Sort-string by name
            self.local_stats
                .filtered_ids
                .sort_by_cached_key(|(id, _, _)| self.index.get(id).map(|note| &note.name));
        } else {
            // all others are usize and can be done in one thing
            self.local_stats.filtered_ids.sort_by_cached_key(
                |(id, global_inlinks, local_inlinks)| {
                    if let Some(note) = self.index.get(id) {
                        match self.sorting {
                            // This should not appear
                            SortingMode::Name => 0,
                            SortingMode::Words => note.words,
                            SortingMode::Chars => note.characters,
                            SortingMode::OutLinks => note.links.len(),
                            SortingMode::GlobalInLinks => *global_inlinks,
                            SortingMode::LocalInLinks => *local_inlinks,
                        }
                    } else {
                        0
                    }
                },
            )
        }

        // Potentially reverse sorting
        if self.sorting_rev {
            self.local_stats.filtered_ids.reverse();
        }

        // Always select the first element whenever a resort is triggered.
        self.selected = 0;
    }
}

impl super::Screen for SelectScreen {
    fn update(&mut self, key: KeyEvent) -> Option<crate::ui::input::Message> {
        // Check for mode
        match self.mode {
            // Main mode: Switch to modes, general command
            SelectMode::Select => match key.code {
                // F: Go to filter mode
                KeyCode::Char('f' | 'F') => {
                    self.mode = SelectMode::Filter;
                }
                // C: Clear filter
                KeyCode::Char('c' | 'C') => {
                    self.text_area.select_all();
                    self.text_area.cut();
                    self.filter(Filter::default());
                }
                // Q: Quit application
                KeyCode::Char('q' | 'Q') => return Some(crate::ui::input::Message::Quit),
                // R: Reload
                KeyCode::Char('r' | 'R') => return Some(crate::ui::input::Message::SwitchSelect),
                // T: Change all/any words requirement
                KeyCode::Char('t' | 'T') => {
                    self.all_tags = !self.all_tags;
                    self.filter_by_area();
                    self.style_text_area();
                }
                KeyCode::Char('n' | 'N') => {
                    self.sort(Some(SortingMode::Name));
                }
                KeyCode::Char('w' | 'W') => {
                    self.sort(Some(SortingMode::Words));
                }
                KeyCode::Char('a' | 'A') => {
                    self.sort(Some(SortingMode::Chars));
                }
                KeyCode::Char('o' | 'O') => {
                    self.sort(Some(SortingMode::OutLinks));
                }
                KeyCode::Char('g') => {
                    self.sort(Some(SortingMode::GlobalInLinks));
                }
                KeyCode::Char('l' | 'L') => {
                    self.sort(Some(SortingMode::LocalInLinks));
                }
                // Selection
                // Down
                KeyCode::Char('j' | 'J') => {
                    self.selected = self
                        .selected
                        .saturating_add(1)
                        .min(self.local_stats.filtered_ids.len() - 1);
                }
                // Up
                KeyCode::Char('k' | 'K') => {
                    self.selected = self.selected.saturating_sub(1);
                }
                // To the start
                KeyCode::Char('G') => {
                    self.selected = 0;
                }
                // Open selected item
                KeyCode::Enter => {
                    if let Some((id, _, _)) = self.local_stats.filtered_ids.get(self.selected) {
                        if let Some(note) = self.index.get(id) {
                            return Some(crate::ui::input::Message::SwitchDisplay(
                                note.path.clone(),
                            ));
                        }
                    }
                }
                _ => {}
            },
            // Filter mode: Type in filter values
            SelectMode::Filter => {
                match key.code {
                    // Escape: Back to main mode
                    KeyCode::Esc => {
                        self.mode = SelectMode::Select;
                    }
                    // Enter: Apply filter, back to main mode
                    KeyCode::Enter => {
                        self.filter_by_area();
                        self.mode = SelectMode::Select;
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
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(6),
        ]);

        let [global_stats_area, local_stats_area, filter_area, table_area] = vertical.areas(area);

        let stats_widths = [
            Constraint::Length(20),
            Constraint::Length(16),
            Constraint::Length(20),
            Constraint::Length(16),
            Constraint::Min(0),
        ];

        //  === Global stats ===

        let global_strings = [
            format!("{:7}", self.global_stats.note_count_total),
            format!("{:7}", self.global_stats.word_count_total),
            format!("{:7}", self.global_stats.tag_count_total),
            format!("{:7}", self.global_stats.char_count_total),
            format!("{:7}", self.global_stats.outlinks_total),
            format!("{:7}", self.global_stats.global_inlinks_total),
        ];

        let global_stats_rows = [
            Row::new(vec![
                "Total notes:",
                &global_strings[0],
                "Total words:",
                &global_strings[1],
            ]),
            Row::new(vec![
                "Total unique tags:",
                &global_strings[2],
                "Total characters:",
                &global_strings[3],
            ]),
            Row::new(vec![
                "Outgoing links:",
                &global_strings[4],
                "Incoming links:",
                &global_strings[5],
            ]),
        ];

        let global_stats = widgets::Table::new(global_stats_rows, stats_widths)
            .column_spacing(1)
            .block(Block::bordered().title("Global Statistics".set_style(self.styles.title_style)));

        //  === Local stats ===

        let local_strings = [
            format!(
                "{:7} ({:02}%)",
                self.local_stats.note_count_total,
                self.local_stats.note_count_total * 100 / self.global_stats.note_count_total
            ),
            format!(
                "{:7} ({:02}%)",
                self.local_stats.word_count_total,
                self.local_stats.word_count_total * 100 / self.global_stats.word_count_total
            ),
            format!(
                "{:7} ({:02}%)",
                self.local_stats.tag_count_total,
                self.local_stats.tag_count_total * 100 / self.global_stats.tag_count_total
            ),
            format!(
                "{:7} ({:02}%)",
                self.local_stats.char_count_total,
                self.local_stats.char_count_total * 100 / self.global_stats.char_count_total
            ),
            format!(
                "{:7} ({:02}%)",
                self.local_stats.outlinks_total,
                self.local_stats.outlinks_total * 100 / self.global_stats.outlinks_total
            ),
            format!(
                "{:7} ({:02}%)",
                self.local_stats.global_inlinks_total,
                self.local_stats.global_inlinks_total * 100
                    / self.global_stats.global_inlinks_total
            ),
        ];

        let local_stats_rows = [
            Row::new(vec![
                "Total notes:",
                &local_strings[0],
                "Total words:",
                &local_strings[1],
            ]),
            Row::new(vec![
                "Total unique tags:",
                &local_strings[2],
                "Total characters:",
                &local_strings[3],
            ]),
            Row::new(vec![
                "Outgoing links:",
                &local_strings[4],
                "Incoming links:",
                &local_strings[5],
            ]),
        ];

        let local_stats = widgets::Table::new(local_stats_rows, stats_widths)
            .column_spacing(1)
            .block(Block::bordered().title("Local Statistics".set_style(self.styles.title_style)));

        // === Filter area ===
        // Mostly styled on creation
        let filter_input = self.text_area.widget();

        // === Table Area ===

        let notes_table_widths = [
            Constraint::Min(25),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ];

        let top_ind = self.selected.saturating_sub(5);

        let mut state = TableState::new()
            .with_offset(0)
            .with_selected(match self.mode {
                SelectMode::Select => Some(self.selected - top_ind),
                SelectMode::Filter => None,
            });

        let notes_rows = self
            .local_stats
            .filtered_ids
            .iter()
            .skip(top_ind)
            .take(table_area.height as usize)
            .map(|(id, global_inlinks, local_inlinks)| {
                self.index.get(id).map(|note| {
                    Row::new(vec![
                        note.name.clone(),
                        format!("{:7}", note.words),
                        format!("{:7}", note.characters),
                        format!("{:7}", note.links.len()),
                        format!("{:7}", global_inlinks),
                        format!("{:7}", local_inlinks),
                    ])
                })
            })
            .flatten()
            .collect::<Vec<Row>>();

        let table = Table::new(notes_rows, notes_table_widths)
            .column_spacing(1)
            .header(Row::new(vec![
                Line::from(vec![
                    Span::styled("N", self.styles.hotkey_style),
                    Span::styled("ame", self.styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("W", self.styles.hotkey_style),
                    Span::styled("ords", self.styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("Ch", self.styles.title_style),
                    Span::styled("a", self.styles.hotkey_style),
                    Span::styled("rs", self.styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("O", self.styles.hotkey_style),
                    Span::styled("ut", self.styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("G", self.styles.hotkey_style),
                    Span::styled("lobalIn", self.styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("L", self.styles.hotkey_style),
                    Span::styled("ocalIn", self.styles.title_style),
                ]),
            ]))
            .highlight_style(self.styles.selected_style)
            .block(Block::bordered().title("Notes".set_style(self.styles.title_style)));

        // === Rendering ===

        Widget::render(filter_input, filter_area, buf);

        Widget::render(global_stats, global_stats_area, buf);
        Widget::render(local_stats, local_stats_area, buf);

        StatefulWidget::render(table, table_area, buf, &mut state)
    }
}
