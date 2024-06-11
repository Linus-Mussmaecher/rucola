use crate::{data, error, files, ui};
use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};

use tui_textarea::TextArea;

/// Describes the current mode of the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SelectMode {
    /// Selecting a note from the list.
    #[default]
    Select,
    /// File managment submenu
    SubmenuFile,
    /// Sorting submenu
    SubmenuSorting,
    /// Typing into the filter box.
    Filter,
    /// Typing into the create box.
    Create,
    /// Typing into the create box to rename a note.
    Rename,
    /// Typing into the create box to move a note.
    Move,
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
    Broken,
}
/// Describes when to show a which stats area.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub enum StatsShow {
    // Always shows both stats
    #[default]
    Both,
    // Shows local stats when filtering and nothing otherwise
    Relevant,
    // Always shows only local stats
    Local,
}

/// The select screen shows the user statistical information about their notes and allows them to select one for display.
pub struct SelectScreen {
    // === DATA ===
    /// A reference to the index of all notes
    index: data::NoteIndexContainer,
    /// The currently displayed statistics for all notes.
    local_stats: data::EnvironmentStats,
    /// The currently displayed statistics for all notes matching the current filter.
    global_stats: data::EnvironmentStats,

    // === Config ===
    /// The file manager this screen uses to enact the user's file system requests on the file system.
    manager: files::FileManager,
    /// The HtmlBuider this screen uses to continuously build html files.
    builder: files::HtmlBuilder,
    /// The used styles.
    styles: ui::UiStyles,

    // === UI ===
    /// The text area to type in filters.
    filter_area: TextArea<'static>,
    /// The text area used to create new notes.
    create_area: TextArea<'static>,
    /// Current input mode
    mode: SelectMode,
    /// Current state of the list
    ///
    /// This is saved as a simple usize from which the ListState to use with ratatui is constructed in immediate mode.
    /// This allows us to convert only the neccessary notes to ListItems and save some time.
    selected: usize,

    // === Sorting options ===
    /// UI mode wether the user wants the filter conditions to all apply or if any (one of them) is enough.
    any_conditions: bool,
    /// Ui mode for the chosen sorting variant
    sorting: SortingMode,
    /// Sort ascedingly.
    sorting_asc: bool,
    /// How to display the two stats blocks.
    stats_show: StatsShow,
}

impl SelectScreen {
    /// Creates a new stats screen, with no filter applied by default
    pub fn new(
        index: data::NoteIndexContainer,
        manager: files::FileManager,
        builder: files::HtmlBuilder,
        styles: ui::UiStyles,
        stats_show: StatsShow,
    ) -> Self {
        let mut res = Self {
            local_stats: data::EnvironmentStats::new_with_filter(&index, data::Filter::default()),
            global_stats: data::EnvironmentStats::new_with_filter(&index, data::Filter::default()),
            index,
            styles,
            builder,
            manager,
            filter_area: TextArea::default(),
            create_area: TextArea::default(),
            mode: SelectMode::Select,
            any_conditions: false,
            sorting: SortingMode::Name,
            sorting_asc: true,
            selected: 0,
            stats_show,
        };

        res.sort();

        res.style_text_area();

        res
    }

    /// Styling of TextArea extracted from constructor to keep it clean.
    fn style_text_area(&mut self) {
        // === Filter ===

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
            Span::styled("A", self.styles.hotkey_style),
            Span::styled(
                if self.any_conditions { "ny" } else { "ll" },
                self.styles.text_style,
            ),
            Span::styled(" Conditions", self.styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Bottom);

        // Apply default self.styles to the filter area

        self.filter_area.set_style(self.styles.input_style);
        self.filter_area
            .set_cursor_line_style(self.styles.input_style);

        self.filter_area.set_block(
            Block::bordered()
                .title(title_top)
                .title(instructions)
                .title(instructions_bot),
        );

        // === Create ===
        // The title
        let title_top = block::Title::from(Line::from(vec![Span::styled(
            "Enter note name...",
            self.styles.title_style,
        )]));

        // Apply default self.styles to the create area

        self.create_area.set_style(self.styles.input_style);
        self.create_area
            .set_cursor_line_style(self.styles.input_style);

        self.create_area
            .set_block(Block::bordered().title(title_top));
    }

    /// Returns the heights of the global and local stats area with this filter string
    pub fn stats_heights(&self, filter_string: Option<&String>) -> (u16, u16) {
        let filtered = filter_string.map(|s| !s.is_empty()).unwrap_or(false);
        match self.stats_show {
            StatsShow::Both => (5, 6),
            StatsShow::Relevant => {
                if filtered {
                    (0, 6)
                } else {
                    (6, 0)
                }
            }
            StatsShow::Local => (0, 6),
        }
    }

    /// Creates a filter from the current content of the filter area.
    fn filter_from_input(&self) -> data::Filter {
        self.filter_area
            .lines()
            .first()
            .map(|l| data::Filter::new(l, self.any_conditions))
            .unwrap_or_default()
    }

    /// Reloads the displayed statistics, showing stats for only those elements of the index matching the specified filter.
    /// Every filtering neccessarily triggers a non-stable resort.
    fn filter(&mut self, filter: data::Filter) {
        // actual filtering
        self.local_stats = data::EnvironmentStats::new_with_filter(&self.index, filter);
        // reset sorting
        self.sorting_asc = false;
        self.sorting = SortingMode::Score;
        self.sort();
        // on a new filter, select the first element
        self.selected = 0;
    }

    /// Re-creates the global and local stats from the index.
    /// To be performed after file management operations.
    pub fn refresh_env_stats(&mut self) {
        // Refresh global stats
        self.global_stats =
            data::EnvironmentStats::new_with_filter(&self.index, data::Filter::default());
        // Refresh local stats
        self.local_stats =
            data::EnvironmentStats::new_with_filter(&self.index, self.filter_from_input());

        // Refresh sorting
        self.sort();
    }

    /// Sets a new sorting mode and direction.
    /// If it did not match the old one, triggers a resort.
    fn set_mode_and_maybe_sort(&mut self, new_mode: impl Into<Option<SortingMode>>, new_asc: bool) {
        // if the sorting mode has changed, resort and select the first element
        if let Some(new_mode) = new_mode.into() {
            if new_mode != self.sorting {
                self.sorting = new_mode;
                self.sort();
                self.selected = 0;
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
        // Always sort by name first
        self.local_stats
            .filtered_stats
            .sort_by_cached_key(|env_stats| env_stats.id.clone());

        // If the sorting mode is not name, now sort by the actual sorting mode.
        if self.sorting != SortingMode::Name {
            // If sorting in reverse is desired, pre-reverse this, so when reversing again later, the list will still be sub-sorted by name ascendingly.
            if !self.sorting_asc {
                self.local_stats.filtered_stats.reverse();
            }
            // all others are usize and can be done in one thing
            self.local_stats
                .filtered_stats
                .sort_by_cached_key(|env_stats| {
                    if let Some(note) = self.index.borrow().get(&env_stats.id) {
                        match self.sorting {
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
    }

    fn extract_string_and_clear(area: &mut TextArea<'static>) -> Option<String> {
        let res = area.lines().first().cloned();
        area.select_all();
        area.cut();
        res
    }
}

impl super::Screen for SelectScreen {
    fn update(&mut self, key: crossterm::event::KeyEvent) -> error::Result<ui::Message> {
        // Check for mode
        match self.mode {
            // Main mode: Switch to modes, general command
            SelectMode::Select => match key.code {
                // Q: Quit application
                KeyCode::Char('q' | 'Q') => return Ok(ui::Message::Quit),
                // R: Got to file management submenu
                KeyCode::Char('m' | 'M') => {
                    self.mode = SelectMode::SubmenuFile;
                }
                // S: Got to sorting submenu
                KeyCode::Char('s' | 'S') => {
                    self.mode = SelectMode::SubmenuSorting;
                }
                // F: Go to filter mode
                KeyCode::Char('f' | 'F') => {
                    self.mode = SelectMode::Filter;
                }
                // C: Clear filter
                KeyCode::Char('c' | 'C') => {
                    let _ = Self::extract_string_and_clear(&mut self.filter_area);
                    self.filter(data::Filter::default());
                }
                // T: Change all/any words requirement
                KeyCode::Char('a' | 'A') => {
                    self.any_conditions = !self.any_conditions;
                    self.filter(self.filter_from_input());
                    self.style_text_area();
                }
                // Selection
                // Down
                KeyCode::Char('j' | 'J') | KeyCode::Down => {
                    self.selected = self
                        .selected
                        .saturating_add(1)
                        .min(self.local_stats.filtered_stats.len().saturating_sub(1));
                }
                // Up
                KeyCode::Char('k' | 'K') | KeyCode::Up => {
                    self.selected = self.selected.saturating_sub(1);
                }
                // To the start
                KeyCode::Char('0') => {
                    self.selected = 0;
                }
                // Open selected item in display view
                KeyCode::Enter | KeyCode::Char('l' | 'L') | KeyCode::Right => {
                    if let Some(env_stats) = self.local_stats.filtered_stats.get(self.selected) {
                        return Ok(ui::Message::DisplayStackPush(env_stats.id.clone()));
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
                        self.filter(self.filter_from_input());
                    }
                    // All other key events are passed on to the text area, then the filter is immediately applied
                    _ => {
                        // Else -> Pass on to the text area
                        self.filter_area.input(key);
                        self.filter(self.filter_from_input());
                    }
                };
            }
            // File mode: Wait for second input
            SelectMode::SubmenuFile => {
                match key.code {
                    // D: Delete note
                    KeyCode::Char('d' | 'D') => {
                        // Get selected element, extract its id
                        if let Some(env_stats) = self.local_stats.filtered_stats.get(self.selected)
                        {
                            // delete it from index & filesystem
                            self.manager
                                .delete_note_file(&mut self.index, &env_stats.id)?;
                            // if successfull, refresh the ui
                            self.refresh_env_stats();
                        }
                        self.mode = SelectMode::Select;
                    }
                    // Open selected item in editor
                    KeyCode::Char('e' | 'E') => {
                        self.mode = SelectMode::Select;
                        if let Some(res) = self
                            // get the selected item in the list for the id
                            .local_stats
                            .filtered_stats
                            .get(self.selected)
                            // use this id in the index to get the note
                            .and_then(|env_stats| {
                                // use the id to get the path
                                self.index.borrow().get(&env_stats.id).map(|note| {
                                    // use the config to create a valid opening command
                                    note.path.clone()
                                })
                            })
                        {
                            return Ok(ui::Message::OpenExternalCommand(
                                self.manager.create_edit_command(&res)?,
                            ));
                        }
                    }
                    // N: Create note
                    KeyCode::Char('n' | 'N') => {
                        self.mode = SelectMode::Create;
                    }
                    // R: Rename note
                    KeyCode::Char('r' | 'R') => {
                        self.mode = SelectMode::Rename;
                    }
                    // M: Move note
                    KeyCode::Char('m' | 'M') => {
                        self.mode = SelectMode::Move;
                    }
                    // Open view mode
                    KeyCode::Char('v' | 'V') => {
                        self.mode = SelectMode::Select;
                        if let Some(env_stats) = self.local_stats.filtered_stats.get(self.selected)
                        {
                            if let Some(note) = self.index.borrow().get(&env_stats.id) {
                                self.builder.create_html(note, true)?;
                                return Ok(ui::Message::OpenExternalCommand(
                                    self.builder.create_view_command(note)?,
                                ));
                            }
                        }
                    }
                    // Back to select mode
                    KeyCode::Esc => {
                        self.mode = SelectMode::Select;
                    }
                    _ => {}
                }
            }
            // Modes that require input in the text box.
            SelectMode::Create | SelectMode::Rename | SelectMode::Move => {
                match key.code {
                    // Escape: Back to main mode, clear the buffer
                    KeyCode::Esc => {
                        let _ = Self::extract_string_and_clear(&mut self.create_area);
                        self.mode = SelectMode::Select;
                    }
                    // Enter: Create note, back to main mode, clear the buffer
                    KeyCode::Enter => {
                        // Switch back to base mode
                        let mode = std::mem::replace(&mut self.mode, SelectMode::Select);
                        // Here, we need to check which mode we are in again
                        match mode {
                            SelectMode::Create => {
                                // Create & register the note
                                self.manager
                                    .create_note_file(Self::extract_string_and_clear(
                                        &mut self.create_area,
                                    ))?;
                                // if successfull, refresh the ui
                                self.refresh_env_stats();
                            }
                            SelectMode::Rename => {
                                // Get the id of currently selected, then delegate to note_file::rename.
                                if let Some(env_stats) =
                                    self.local_stats.filtered_stats.get(self.selected)
                                {
                                    self.manager.rename_note_file(
                                        &mut self.index,
                                        &env_stats.id,
                                        Self::extract_string_and_clear(&mut self.create_area),
                                    )?;
                                    // if successfull, refresh the ui
                                    self.refresh_env_stats();
                                }
                            }
                            SelectMode::Move => {
                                // Get the id of currently selected, then delegate to note_file::move.
                                if let Some(env_stats) =
                                    self.local_stats.filtered_stats.get(self.selected)
                                {
                                    self.manager.move_note_file(
                                        &mut self.index,
                                        &env_stats.id,
                                        Self::extract_string_and_clear(&mut self.create_area),
                                    )?;
                                    // if successfull, refresh the ui
                                    self.refresh_env_stats();
                                }
                            }
                            _ => {
                                //This should NOT happen
                            }
                        }
                    }
                    // All other key events are passed on to the text area
                    _ => {
                        // Else -> Pass on to the text area
                        self.create_area.input(key);
                    }
                };
            }
            // Sorting submenu: Wait for second input
            SelectMode::SubmenuSorting => match key.code {
                KeyCode::Char('a' | 'A') => {
                    self.set_mode_and_maybe_sort(SortingMode::Name, true);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('w' | 'W') => {
                    self.set_mode_and_maybe_sort(SortingMode::Words, false);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('c' | 'C') => {
                    self.set_mode_and_maybe_sort(SortingMode::Chars, false);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('o' | 'O') => {
                    self.set_mode_and_maybe_sort(SortingMode::GlobalOutLinks, false);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('u' | 'U') => {
                    self.set_mode_and_maybe_sort(SortingMode::LocalOutLinks, false);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('i' | 'I') => {
                    self.set_mode_and_maybe_sort(SortingMode::GlobalInLinks, false);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('n' | 'N') => {
                    self.set_mode_and_maybe_sort(SortingMode::LocalInLinks, false);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('b' | 'B') => {
                    self.set_mode_and_maybe_sort(SortingMode::Broken, false);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Char('r' | 'R') => {
                    self.set_mode_and_maybe_sort(None, !self.sorting_asc);
                    self.mode = SelectMode::Select;
                }
                KeyCode::Esc | KeyCode::Char('s' | 'S') => {
                    self.mode = SelectMode::Select;
                }
                _ => {}
            },
        };

        Ok(ui::Message::None)
    }

    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer) {
        // Get the filter string (neccssary to determine if a filter is active)
        let (global_size, local_size) = self.stats_heights(self.filter_area.lines().last());
        // Vertical layout
        let vertical = Layout::vertical([
            Constraint::Length(global_size),
            Constraint::Length(local_size),
            Constraint::Length(3),
            Constraint::Min(6),
        ]);

        // Generate areas
        let [global_stats_area, local_stats_area, filter_area, table_area] = vertical.areas(area);

        // Horizontal layout of upper boxes
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
                Cell::from("Total notes:"),
                Cell::from(format!("{:7}", self.global_stats.note_count_total)),
                Cell::from("Total words:"),
                Cell::from(format!("{:7}", self.global_stats.word_count_total)),
            ]),
            Row::new(vec![
                Cell::from("Total unique tags:"),
                Cell::from(format!("{:7}", self.global_stats.tag_count_total)),
                Cell::from("Total characters:"),
                Cell::from(format!("{:7}", self.global_stats.char_count_total)),
            ]),
            Row::new(vec![
                Cell::from("Total links:"),
                Cell::from(format!("{:7}", self.global_stats.local_local_links)),
                Cell::from("Broken links:"),
                Cell::from(format!("{:7}", self.global_stats.broken_links)),
            ]),
        ];

        // Finalize global table from the row data and the widths
        let global_stats = Table::new(global_stats_rows, stats_widths)
            .column_spacing(1)
            .block(Block::bordered().title("Global Statistics".set_style(self.styles.title_style)));

        //  === Local stats ===

        let local_stats_rows = [
            Row::new(vec![
                Cell::from("Total notes:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.note_count_total,
                    self.local_stats.note_count_total * 100
                        / self.global_stats.note_count_total.max(1)
                )),
                Cell::from("Total words:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.word_count_total,
                    self.local_stats.word_count_total * 100
                        / self.global_stats.word_count_total.max(1)
                )),
            ]),
            Row::new(vec![
                Cell::from("Total unique tags:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.tag_count_total,
                    self.local_stats.tag_count_total * 100
                        / self.global_stats.tag_count_total.max(1)
                )),
                Cell::from("Total characters:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.char_count_total,
                    self.local_stats.char_count_total * 100
                        / self.global_stats.char_count_total.max(1)
                )),
            ]),
            Row::new(vec![
                Cell::from("Incoming links:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.global_local_links,
                    self.local_stats.global_local_links * 100
                        / self.global_stats.local_local_links.max(1),
                )),
                Cell::from("Outgoing links:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.local_global_links,
                    self.local_stats.local_global_links * 100
                        / self.global_stats.local_local_links.max(1),
                )),
            ]),
            Row::new(vec![
                Cell::from("Internal links:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.local_local_links,
                    self.local_stats.local_local_links * 100
                        / self.global_stats.local_local_links.max(1),
                )),
                Cell::from("Broken links:"),
                Cell::from(format!(
                    "{:7} ({:3}%)",
                    self.local_stats.broken_links,
                    self.local_stats.broken_links * 100 / self.global_stats.broken_links.max(1)
                )),
            ]),
        ];

        // Finalize local table
        let local_stats = Table::new(local_stats_rows, stats_widths)
            .column_spacing(1)
            .block(Block::bordered().title("Local Statistics".set_style(self.styles.title_style)));

        // === Filter area ===

        // Mostly styled on creation
        let filter_input = self.filter_area.widget();

        // === Table Area ===

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

        // Generate state from selected element
        let mut state = TableState::new()
            .with_offset(
                self.selected
                    // try to keep element at above 1/3rd of the total height
                    .saturating_sub(table_area.height as usize / 3)
                    .min(
                        // but when reaching the end of the list, still scroll down
                        self.local_stats
                            .filtered_stats
                            .len()
                            // correct for table edges
                            .saturating_add(3)
                            .saturating_sub(table_area.height as usize),
                    ),
            )
            // In certain modes, show a selected element
            .with_selected(match self.mode {
                SelectMode::Select
                | SelectMode::Rename
                | SelectMode::Move
                | SelectMode::SubmenuFile
                | SelectMode::SubmenuSorting => Some(self.selected),
                SelectMode::Filter | SelectMode::Create => None,
            });

        // Generate row data
        let notes_rows = self
            .local_stats
            .filtered_stats
            .iter()
            .flat_map(|env_stats| {
                // generate the stats row for each element
                self.index.borrow().get(&env_stats.id).map(|note| {
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
            .collect::<Vec<Row>>();

        // Instructions at the bottom of the page
        let instructions_bot_left = block::Title::from(Line::from(vec![
            Span::styled("J", self.styles.hotkey_style),
            Span::styled("/", self.styles.text_style),
            Span::styled("", self.styles.hotkey_style),
            Span::styled(": Down──", self.styles.text_style),
            Span::styled("K", self.styles.hotkey_style),
            Span::styled("/", self.styles.text_style),
            Span::styled("", self.styles.hotkey_style),
            Span::styled(": Up──", self.styles.text_style),
            Span::styled("L", self.styles.hotkey_style),
            Span::styled("/", self.styles.text_style),
            Span::styled("", self.styles.hotkey_style),
            Span::styled("/", self.styles.text_style),
            Span::styled("󰌑", self.styles.hotkey_style),
            Span::styled(": Open──", self.styles.text_style),
        ]))
        .alignment(Alignment::Left)
        .position(block::Position::Bottom);

        let instructions_bot_right = block::Title::from(Line::from(vec![
            Span::styled("M", self.styles.hotkey_style),
            Span::styled("anage Files──", self.styles.text_style),
            Span::styled("S", self.styles.hotkey_style),
            Span::styled("orting──", self.styles.text_style),
            Span::styled("Q", self.styles.hotkey_style),
            Span::styled("uit", self.styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Bottom);

        let table_heading_key_style = if self.mode == SelectMode::SubmenuSorting {
            self.styles.hotkey_style
        } else {
            self.styles.subtitle_style
        };

        // Finally generate the table from the generated row and width data
        let table = Table::new(notes_rows, notes_table_widths)
            .column_spacing(1)
            // Add Headers
            .header(Row::new(vec![
                Line::from(vec![
                    Span::styled("N", self.styles.subtitle_style),
                    Span::styled("a", table_heading_key_style),
                    Span::styled("me", self.styles.subtitle_style),
                ]),
                Line::from(vec![
                    Span::styled("  ", self.styles.subtitle_style),
                    Span::styled("W", table_heading_key_style),
                    Span::styled("ords", self.styles.subtitle_style),
                ]),
                Line::from(vec![
                    Span::styled("  ", self.styles.subtitle_style),
                    Span::styled("C", table_heading_key_style),
                    Span::styled("hars", self.styles.subtitle_style),
                ]),
                Line::from(vec![
                    Span::styled("Global", self.styles.subtitle_style),
                    Span::styled("O", table_heading_key_style),
                    Span::styled("ut", self.styles.subtitle_style),
                ]),
                Line::from(vec![
                    Span::styled("LocalO", self.styles.subtitle_style),
                    Span::styled("u", table_heading_key_style),
                    Span::styled("t", self.styles.subtitle_style),
                ]),
                Line::from(vec![
                    Span::styled("Global", self.styles.subtitle_style),
                    Span::styled("I", table_heading_key_style),
                    Span::styled("n", self.styles.subtitle_style),
                ]),
                Line::from(vec![
                    Span::styled("LocalI", self.styles.subtitle_style),
                    Span::styled("n", table_heading_key_style),
                ]),
            ]))
            .highlight_style(self.styles.selected_style)
            // Add Instructions and a title
            .block(
                Block::bordered()
                    .title("Notes".set_style(self.styles.title_style))
                    .title(instructions_bot_left)
                    .title(instructions_bot_right),
            );

        // === Rendering ===

        Widget::render(filter_input, filter_area, buf);

        Widget::render(global_stats, global_stats_area, buf);
        Widget::render(local_stats, local_stats_area, buf);

        StatefulWidget::render(table, table_area, buf, &mut state);

        // Render eventual pop-ups
        match self.mode {
            SelectMode::SubmenuFile | SelectMode::SubmenuSorting => {
                let contents = if self.mode == SelectMode::SubmenuFile {
                    vec![
                        ("N", "New note"),
                        ("E", "Edit selected note"),
                        ("R", "Rename selected note"),
                        ("M", "Move selected note"),
                        ("D", "Delete selected note"),
                        ("V", "Open HTML in external viewer"),
                    ]
                } else {
                    vec![
                        ("A", "Sort by name"),
                        ("W", "Sort by words"),
                        ("C", "Sort by characters"),
                        ("O", "Sort by global outlinks"),
                        ("U", "Sort by local outlinks"),
                        ("I", "Sort by global inlinks"),
                        ("N", "Sort by local inlinks"),
                        ("B", "Sort by broken links"),
                        ("R", "Reverse sorting"),
                    ]
                };

                let popup_areas = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(contents.len() as u16 + 2),
                    Constraint::Length(1),
                ])
                .split(area);

                let br_area = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(
                        contents
                            .iter()
                            .map(|(_key, desc)| desc.len())
                            .max()
                            .unwrap_or_default() as u16
                            + 5,
                    ),
                    Constraint::Length(1),
                ])
                .split(popup_areas[1])[1];

                let rows = contents
                    .into_iter()
                    .map(|(key, description)| {
                        Row::new(vec![
                            Span::styled(key, self.styles.hotkey_style),
                            Span::styled(description, self.styles.text_style),
                        ])
                    })
                    .collect::<Vec<_>>();

                let widths = [Constraint::Length(2), Constraint::Fill(1)];

                let popup_table = Table::new(rows, widths)
                    .block(Block::bordered())
                    .column_spacing(1);

                // Clear the area and then render the widget on top.
                Widget::render(Clear, br_area, buf);
                Widget::render(popup_table, br_area, buf);
            }
            SelectMode::Filter | SelectMode::Select => {}
            SelectMode::Create | SelectMode::Rename | SelectMode::Move => {
                let create_input = self.create_area.widget();

                let popup_areas = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .split(area);

                let center_area = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Percentage(60),
                    Constraint::Fill(1),
                ])
                .split(popup_areas[1])[1];

                // Clear the area and then render the widget on top.
                Widget::render(Clear, center_area, buf);
                Widget::render(create_input, center_area, buf);
            }
        }
    }
}
