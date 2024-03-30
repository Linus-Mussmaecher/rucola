use crate::{config, data, ui};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

use std::{collections::HashMap, rc::Rc};
use tui_textarea::TextArea;

/// Describes the current mode of the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SelectMode {
    /// Selecting a note from the list.
    #[default]
    Select,
    /// Typing into the filter box.
    Filter,
}

/// Describes the current sorting mode of the displayed list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SortingMode {
    #[default]
    Name,
    Words,
    Chars,
    GlobalOutLinks,
    LocalOutLinks,
    GlobalInLinks,
    LocalInLinks,
    Score,
}

/// The select screen shows the user statistical information about their notes and allows them to select one for display.
pub struct SelectScreen {
    // === Displayed Data ===
    /// A reference to the index of all notes
    index: Rc<HashMap<String, data::Note>>,
    /// The currently displayed statistics for all notes.
    local_stats: data::EnvironmentStats,
    /// The currently displayed statistics for all notes matching the current filter.
    global_stats: data::EnvironmentStats,
    /// The styles used on this screen.
    styles: ui::UiStyles,

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
    /// Sort ascedingly
    sorting_asc: bool,
    /// Refilter while typing or not (to save resources)
    dynamic_filter: bool,
}

impl SelectScreen {
    /// Creates a new stats screen, with no filter applied by default
    pub fn new(index: Rc<HashMap<String, data::Note>>, config: &config::Config) -> Self {
        let mut res = Self {
            local_stats: data::EnvironmentStats::new_with_filters(&index, data::Filter::default()),
            global_stats: data::EnvironmentStats::new_with_filters(&index, data::Filter::default()),
            index,
            text_area: TextArea::default(),
            mode: SelectMode::Select,
            styles: config.get_ui_styles().clone(),
            all_tags: false,
            sorting: SortingMode::Name,
            sorting_asc: false,
            selected: 0,
            dynamic_filter: config.get_dynamic_filter(),
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

    /// Creates a filter from the current content of the text area.
    fn filter_from_input(&self) -> data::Filter {
        // We should only have one line, read that one
        if let Some(line) = self.text_area.lines().first() {
            let mut filter = data::Filter::default();
            // default filter is this line with all white space removed
            filter.title = line.chars().filter(|c| !c.is_whitespace()).collect();

            // Go through words
            for word in line.split_whitespace() {
                if word.starts_with('#') {
                    // words with a hash count as a tag
                    filter.tags.push(word.to_string());
                    // and remove it from the title to match
                    filter.title = filter.title.replace(&word, "");
                }
            }

            // check for any or all tags
            filter.all_tags = self.all_tags;

            filter
        } else {
            data::Filter::default()
        }
    }

    /// Reloads the displayed statistics, showing stats for only those elements of the index matching the specified filter.
    /// Every filtering neccessarily triggers a non-stable resort.
    fn filter(&mut self, filter: data::Filter) {
        self.local_stats = data::EnvironmentStats::new_with_filters(&self.index, filter);
        self.sorting_asc = false;
        self.sorting = SortingMode::Score;
        self.sort();
    }

    /// Sets a new sorting mode and direction.
    /// If it did not match the old one, triggers a resort.
    fn set_mode_and_maybe_sort(&mut self, new_mode: impl Into<Option<SortingMode>>, new_asc: bool) {
        // if the sorting mode has changed, resort
        if let Some(new_mode) = new_mode.into() {
            if new_mode != self.sorting {
                self.sorting = new_mode;
                self.sort();
            }
        }
        // if the asc has changed, reverse
        if new_asc != self.sorting_asc {
            self.sorting_asc = new_asc;
            self.local_stats.filtered_stats.reverse();
        }
    }

    /// Sorts the note display according to the current sorting mode.
    fn sort(&mut self) {
        if self.sorting == SortingMode::Name {
            // Name: Sort-string by name
            self.local_stats
                .filtered_stats
                .sort_by_cached_key(|env_stats| env_stats.id.clone());
        } else {
            // all others are usize and can be done in one thing
            self.local_stats
                .filtered_stats
                .sort_by_cached_key(|env_stats| {
                    if let Some(note) = self.index.get(&env_stats.id) {
                        match self.sorting {
                            // This should not appear
                            SortingMode::Name => 0,
                            SortingMode::Words => note.words,
                            SortingMode::Chars => note.characters,
                            SortingMode::GlobalOutLinks => env_stats.outlinks_global,
                            SortingMode::LocalOutLinks => env_stats.outlinks_local,
                            SortingMode::GlobalInLinks => env_stats.inlinks_global,
                            SortingMode::LocalInLinks => env_stats.inlinks_local,
                            SortingMode::Score => env_stats.match_score as usize,
                        }
                    } else {
                        0
                    }
                })
        }

        // Potentially reverse sorting
        if !self.sorting_asc {
            self.local_stats.filtered_stats.reverse();
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
                    self.filter(data::Filter::default());
                }
                // Q: Quit application
                KeyCode::Char('q' | 'Q') => return Some(crate::ui::input::Message::Quit),
                // T: Change all/any words requirement
                KeyCode::Char('t' | 'T') => {
                    self.all_tags = !self.all_tags;
                    self.filter(self.filter_from_input());
                    self.style_text_area();
                }
                // R: Reverse sorting
                KeyCode::Char('s' | 'S') => {
                    self.set_mode_and_maybe_sort(None, !self.sorting_asc);
                }
                KeyCode::Char('n' | 'N') => {
                    self.set_mode_and_maybe_sort(SortingMode::Name, true);
                }
                KeyCode::Char('w' | 'W') => {
                    self.set_mode_and_maybe_sort(SortingMode::Words, false);
                }
                KeyCode::Char('a' | 'A') => {
                    self.set_mode_and_maybe_sort(SortingMode::Chars, false);
                }
                KeyCode::Char('o' | 'O') => {
                    self.set_mode_and_maybe_sort(SortingMode::GlobalOutLinks, false);
                }
                KeyCode::Char('u' | 'U') => {
                    self.set_mode_and_maybe_sort(SortingMode::LocalOutLinks, false);
                }
                KeyCode::Char('g') => {
                    self.set_mode_and_maybe_sort(SortingMode::GlobalInLinks, false);
                }
                KeyCode::Char('l' | 'L') => {
                    self.set_mode_and_maybe_sort(SortingMode::LocalInLinks, false);
                }
                // Selection
                // Down
                KeyCode::Char('j' | 'J') | KeyCode::Down => {
                    self.selected = self
                        .selected
                        .saturating_add(1)
                        .min(self.local_stats.filtered_stats.len() - 1);
                }
                // Up
                KeyCode::Char('k' | 'K') | KeyCode::Up => {
                    self.selected = self.selected.saturating_sub(1);
                }
                // To the start
                KeyCode::Char('G') => {
                    self.selected = 0;
                }
                // Open selected item
                KeyCode::Enter => {
                    if let Some(env_stats) = self.local_stats.filtered_stats.get(self.selected) {
                        if let Some(note) = self.index.get(&env_stats.id) {
                            return Some(crate::ui::input::Message::SwitchDisplay(note.clone()));
                        }
                    }
                }
                _ => {}
            },
            // Filter mode: Type in filter values
            SelectMode::Filter => {
                match key.code {
                    // Escape or Enter: Back to main mode
                    KeyCode::Esc | KeyCode::Enter => {
                        self.mode = SelectMode::Select;
                        if key.code == KeyCode::Enter && !self.dynamic_filter {
                            self.filter(self.filter_from_input());
                        }
                    }
                    // All other key events are passed on to the text area, then the filter is immediately applied
                    _ => {
                        // Else -> Pass on to the text area
                        self.text_area.input_without_shortcuts(key);
                        if self.dynamic_filter {
                            self.filter(self.filter_from_input());
                        }
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
            Constraint::Length(6),
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
            format!("{:7}", self.global_stats.local_local_links),
            format!("{:7}", self.global_stats.broken_links),
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
                "Total links:",
                &global_strings[4],
                "Broken links:",
                &global_strings[5],
            ]),
        ];

        let global_stats = Table::new(global_stats_rows, stats_widths)
            .column_spacing(1)
            .block(Block::bordered().title("Global Statistics".set_style(self.styles.title_style)));

        //  === Local stats ===

        let local_strings = [
            format!(
                "{:7} ({:3}%)",
                self.local_stats.note_count_total,
                self.local_stats.note_count_total * 100 / self.global_stats.note_count_total.max(1)
            ),
            format!(
                "{:7} ({:3}%)",
                self.local_stats.word_count_total,
                self.local_stats.word_count_total * 100 / self.global_stats.word_count_total.max(1)
            ),
            format!(
                "{:7} ({:3}%)",
                self.local_stats.tag_count_total,
                self.local_stats.tag_count_total * 100 / self.global_stats.tag_count_total.max(1)
            ),
            format!(
                "{:7} ({:3}%)",
                self.local_stats.char_count_total,
                self.local_stats.char_count_total * 100 / self.global_stats.char_count_total.max(1)
            ),
            format!(
                "{:7} ({:3}%)",
                self.local_stats.global_local_links,
                self.local_stats.global_local_links * 100
                    / self.global_stats.global_local_links.max(1)
            ),
            format!(
                "{:7} ({:3}%)",
                self.local_stats.local_global_links,
                self.local_stats.local_global_links * 100
                    / self.global_stats.local_global_links.max(1)
            ),
            format!(
                "{:7} ({:3}%)",
                self.local_stats.local_local_links,
                self.local_stats.local_local_links * 100
                    / self.global_stats.local_local_links.max(1)
            ),
            format!(
                "{:7} ({:3}%)",
                self.local_stats.broken_links,
                self.local_stats.broken_links * 100 / self.global_stats.broken_links.max(1)
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
                "Incoming links:",
                &local_strings[4],
                "Outgoing links:",
                &local_strings[5],
            ]),
            Row::new(vec![
                "Internal links:",
                &local_strings[6],
                "Broken links:",
                &local_strings[7],
            ]),
        ];

        let local_stats = Table::new(local_stats_rows, stats_widths)
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
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
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
            .filtered_stats
            .iter()
            .skip(top_ind)
            .take(table_area.height as usize)
            .map(|env_stats| {
                self.index.get(&env_stats.id).map(|note| {
                    Row::new(vec![
                        note.name.clone(),
                        format!("{:7}", note.words),
                        format!("{:7}", note.characters),
                        format!("{:7}", env_stats.outlinks_global),
                        format!("{:7}", env_stats.outlinks_local),
                        format!("{:7}", env_stats.inlinks_global),
                        format!("{:7}", env_stats.inlinks_local),
                    ])
                })
            })
            .flatten()
            .collect::<Vec<Row>>();

        let instructions_bot = block::Title::from(Line::from(vec![
            Span::styled(
                if self.sorting_asc {
                    "Ascending "
                } else {
                    "Descending "
                },
                self.styles.text_style,
            ),
            Span::styled("S", self.styles.hotkey_style),
            Span::styled("orting", self.styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Bottom);

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
                    Span::styled("Global", self.styles.title_style),
                    Span::styled("O", self.styles.hotkey_style),
                    Span::styled("ut", self.styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("LocalO", self.styles.title_style),
                    Span::styled("u", self.styles.hotkey_style),
                    Span::styled("t", self.styles.title_style),
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
            .block(
                Block::bordered()
                    .title("Notes".set_style(self.styles.title_style))
                    .title(instructions_bot),
            );

        // === Rendering ===

        Widget::render(filter_input, filter_area, buf);

        Widget::render(global_stats, global_stats_area, buf);
        Widget::render(local_stats, local_stats_area, buf);

        StatefulWidget::render(table, table_area, buf, &mut state)
    }
}
