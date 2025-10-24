use crate::{data, error, io, ui};
use itertools::Itertools;
use ratatui::crossterm::event::KeyCode;
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
    /// Show the help screen for the filter box.
    FilterHelp,
    /// Typing into the create box.
    Create,
    /// Typing into the create box to rename a note.
    Rename,
    /// Typing into the create box to move a note.
    Move,
    /// Confirmation for deletion
    Delete,
}

/// Describes when to show a which stats area.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Default)]
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
    manager: io::FileManager,
    /// The git repository the vault is stored in, if any.
    git_manager: Option<io::GitManager>,
    /// The HtmlBuider this screen uses to continuously build html files.
    builder: io::HtmlBuilder,
    /// The used styles.
    styles: ui::UiStyles,

    // === UI ===
    /// The text area to type in filters.
    filter_area: TextArea<'static>,
    /// The text area used to create new notes.
    name_area: TextArea<'static>,
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
    /// Default sorting of the note list.
    sorting: data::SortingMode,
    /// Default sorting direction of the note list.
    sorting_asc: bool,
    /// How to display the two stats blocks.
    stats_show: StatsShow,
}

impl SelectScreen {
    /// Creates a new stats screen, with no filter applied by default
    pub fn new(
        index: data::NoteIndexContainer,
        manager: io::FileManager,
        git_manager: Option<io::GitManager>,
        builder: io::HtmlBuilder,
        styles: ui::UiStyles,
        config: &crate::Config,
    ) -> Self {
        let mut res = Self {
            local_stats: data::EnvironmentStats::new_with_filter(&index, data::Filter::default()),
            global_stats: data::EnvironmentStats::new_with_filter(&index, data::Filter::default()),
            index: index.clone(),
            styles,
            builder,
            manager,
            git_manager,
            filter_area: TextArea::default(),
            name_area: TextArea::default(),
            mode: SelectMode::Select,
            any_conditions: false,
            sorting: config.default_sorting,
            sorting_asc: config.default_sorting_asc,
            selected: 0,
            stats_show: config.stats_show,
        };

        res.local_stats
            .sort(index, config.default_sorting, config.default_sorting_asc);

        res.style_text_area();

        res
    }

    /// Styling of TextArea extracted from constructor to keep it clean.
    fn style_text_area(&mut self) {
        // === Filter ===

        // The actual title
        let title_top = Line::from(vec![
            Span::styled("F", self.styles.hotkey_style),
            Span::styled("ilter", self.styles.title_style),
        ])
        .left_aligned();

        // The hotkey instructions at the bottom.
        let instructions = Line::from(vec![
            Span::styled("C", self.styles.hotkey_style),
            Span::styled("lear filter", self.styles.text_style),
        ])
        .right_aligned();

        let instructions_bot = Line::from(vec![
            Span::styled("A", self.styles.hotkey_style),
            Span::styled(
                if self.any_conditions { "ny" } else { "ll" },
                self.styles.text_style,
            ),
            Span::styled(" Conditions──", self.styles.text_style),
            Span::styled("H", self.styles.hotkey_style),
            Span::styled("elp", self.styles.text_style),
        ])
        .right_aligned();

        // Apply default self.styles to the filter area

        self.filter_area.set_style(self.styles.input_style);
        self.filter_area
            .set_cursor_line_style(self.styles.input_style);

        self.filter_area.set_block(
            Block::bordered()
                .title_top(title_top)
                .title_top(instructions)
                .title_bottom(instructions_bot),
        );

        // === Create ===
        // The title
        let title_top = block::Title::from(Line::from(vec![Span::styled(
            "Enter note name...",
            self.styles.title_style,
        )]));

        // Apply default self.styles to the create area

        self.name_area.set_style(self.styles.input_style);
        self.name_area
            .set_cursor_line_style(self.styles.input_style);

        self.name_area.set_block(Block::bordered().title(title_top));
    }

    /// Sets the title & content of the name_area block
    fn set_name_area(&mut self, title: &str, content: Option<String>) {
        let title_top = block::Title::from(Line::from(vec![Span::styled(
            title.to_owned(),
            self.styles.title_style,
        )]));

        self.name_area.set_block(Block::bordered().title(title_top));
        // it is assumed the buffer is empty so far
        if let Some(content) = content {
            self.name_area.insert_str(content);
        }
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
        // // reset sorting
        // self.sorting_asc = false;
        // self.sorting = data::SortingMode::Score;
        self.local_stats
            .sort(self.index.clone(), data::SortingMode::Score, false);
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
        self.local_stats
            .sort(self.index.clone(), self.sorting, self.sorting_asc);
    }
}

impl super::Screen for SelectScreen {
    fn update(&mut self, key: ratatui::crossterm::event::KeyEvent) -> error::Result<ui::Message> {
        let key: ratatui::crossterm::event::KeyEvent = key;
        // Check for mode
        match self.mode {
            // Main mode: Switch to modes, general command
            SelectMode::Select => match key.code {
                // Q: Quit application
                KeyCode::Char('q' | 'Q') => return Ok(ui::Message::Quit),
                // M: Got to file management submenu
                KeyCode::Char('m' | 'M') => {
                    self.mode = SelectMode::SubmenuFile;
                }
                // S: Got to sorting submenu
                KeyCode::Char('s' | 'S') => {
                    self.mode = SelectMode::SubmenuSorting;
                }
                // F: or /: Go to filter mode
                KeyCode::Char('f' | 'F' | '/') => {
                    self.mode = SelectMode::Filter;
                }
                // ?: Go to filter help mode
                KeyCode::Char('?' | 'h' | 'H') => {
                    self.mode = SelectMode::FilterHelp;
                }
                // C: Clear filter
                KeyCode::Char('c' | 'C') => {
                    let _ = super::extract_string_and_clear(&mut self.filter_area);
                    self.filter(data::Filter::default());
                    self.local_stats
                        .sort(self.index.clone(), self.sorting, self.sorting_asc);
                }
                // T: Change all/any words requirement
                KeyCode::Char('a' | 'A') => {
                    self.any_conditions = !self.any_conditions;
                    self.filter(self.filter_from_input());
                    self.style_text_area();
                }
                // Open selected item in editor
                KeyCode::Char('e' | 'E') => {
                    self.mode = SelectMode::Select;
                    if let Some(res) = self
                        // get the selected item in the list for the id
                        .local_stats
                        .get_selected(self.selected)
                        // use this id in the index to get the note
                        .and_then(|env_stats| {
                            // use the id to get the path
                            self.index
                                .borrow()
                                .get(&env_stats.id)
                                .map(|note| note.path.clone())
                        })
                    {
                        // use the config to create a valid opening command
                        return Ok(ui::Message::OpenExternalCommand(Box::new(
                            self.manager.create_edit_command(&res)?,
                        )));
                    }
                }
                // Open view mode
                KeyCode::Char('v' | 'V') => {
                    self.mode = SelectMode::Select;
                    if let Some(env_stats) = self.local_stats.get_selected(self.selected) {
                        if let Some(note) = self.index.borrow().get(&env_stats.id) {
                            self.builder.create_html(note, true)?;
                            return Ok(ui::Message::OpenExternalCommand(Box::new(
                                self.manager
                                    .create_view_command(note, key.code == KeyCode::Char('v'))?,
                            )));
                        }
                    }
                }
                // Selection
                // Down
                KeyCode::Char('j' | 'J') | KeyCode::Down => {
                    self.selected = self
                        .selected
                        .saturating_add(1)
                        .min(self.local_stats.len().saturating_sub(1));
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
                    if let Some(env_stats) = self.local_stats.get_selected(self.selected) {
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
            SelectMode::FilterHelp => {
                match key.code {
                    // Escape or Enter: Back to main mode
                    KeyCode::Esc | KeyCode::Char('c' | 'C') => {
                        self.mode = SelectMode::Select;
                    }
                    // All other key events are ignored
                    _ => {}
                };
            }
            // File mode: Wait for second input
            SelectMode::SubmenuFile => {
                match key.code {
                    // D: Delete note
                    KeyCode::Char('d' | 'D') => {
                        self.mode = SelectMode::Delete;
                    }
                    // N: Create note
                    KeyCode::Char('n' | 'N') => {
                        self.mode = SelectMode::Create;
                        self.set_name_area("Enter name of new note...", None);
                    }
                    // R: Rename note
                    KeyCode::Char('r' | 'R') => {
                        self.mode = SelectMode::Rename;
                        let name = self
                            // get the selected item in the list for the id
                            .local_stats
                            .get_selected(self.selected)
                            // use this id in the index to get the note
                            .and_then(|env_stats| {
                                // use the id to get the name
                                self.index
                                    .borrow()
                                    .get(&env_stats.id)
                                    .map(|note| note.name.clone())
                            });

                        self.set_name_area("Enter new name of note...", name);
                    }
                    // M: Move note
                    KeyCode::Char('m' | 'M') => {
                        self.mode = SelectMode::Move;
                        self.set_name_area("Enter new location relative to vault...", None);
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
                        let _ = super::extract_string_and_clear(&mut self.name_area);
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
                                self.manager.create_note_file(
                                    &super::extract_string_and_clear(&mut self.name_area)
                                        .ok_or_else(|| {
                                            error::RucolaError::Input(String::from(
                                                "New note may not be empty.",
                                            ))
                                        })?,
                                )?;
                                // if successfull, refresh the ui
                                self.index.borrow().poll_file_system();
                                self.refresh_env_stats();
                            }
                            SelectMode::Rename => {
                                // Get the id of currently selected, then delegate to note_file::rename.
                                if let Some(env_stats) =
                                    self.local_stats.get_selected(self.selected)
                                {
                                    self.manager.rename_note_file(
                                        self.index.clone(),
                                        &env_stats.id,
                                        super::extract_string_and_clear(&mut self.name_area)
                                            .ok_or_else(|| {
                                                error::RucolaError::Input(
                                                    "New name is empty.".to_string(),
                                                )
                                            })?,
                                    )?;
                                    // if successfull, update the index and refresh the ui
                                    self.index.borrow().poll_file_system();
                                    self.refresh_env_stats();
                                }
                            }
                            SelectMode::Move => {
                                // Get the id of currently selected, then delegate to note_file::move.
                                if let Some(env_stats) =
                                    self.local_stats.get_selected(self.selected)
                                {
                                    self.manager.move_note_file(
                                        self.index.clone(),
                                        &env_stats.id,
                                        super::extract_string_and_clear(&mut self.name_area)
                                            .ok_or_else(|| {
                                                error::RucolaError::Input(
                                                    "Move target is empty.".to_string(),
                                                )
                                            })?,
                                    )?;
                                    // if successfull, refresh the ui
                                    self.index.borrow().poll_file_system();
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
                        self.name_area.input(key);
                    }
                };
            }
            // Deletion submenu: Enter deletes, all others cancel.
            SelectMode::Delete => match key.code {
                KeyCode::Enter => {
                    if let Some(env_stats) = self
                        // get the selected item in the list for the id
                        .local_stats
                        .get_selected(self.selected)
                    {
                        // delete it from index & filesystem
                        self.manager
                            .delete_note_file(self.index.clone(), &env_stats.id)?;
                        // if successfull, refresh the ui
                        self.index.borrow().poll_file_system();
                        self.refresh_env_stats();
                    }
                    self.mode = SelectMode::Select;
                }
                _ => {
                    self.mode = SelectMode::Select;
                }
            },
            // Sorting submenu: Wait for second input
            SelectMode::SubmenuSorting => {
                let old_selected = self.selected;
                self.selected = 0;
                self.mode = SelectMode::Select;
                match key.code {
                    KeyCode::Char('s' | 'S') => {
                        self.local_stats
                            .sort(self.index.clone(), data::SortingMode::Shuffle, true);
                    }
                    KeyCode::Char('a' | 'A') => {
                        self.local_stats
                            .sort(self.index.clone(), data::SortingMode::Name, true);
                    }
                    KeyCode::Char('w' | 'W') => {
                        self.local_stats
                            .sort(self.index.clone(), data::SortingMode::Words, false);
                    }
                    KeyCode::Char('c' | 'C') => {
                        self.local_stats
                            .sort(self.index.clone(), data::SortingMode::Chars, false);
                    }
                    KeyCode::Char('o' | 'O') => {
                        self.local_stats.sort(
                            self.index.clone(),
                            data::SortingMode::GlobalOutLinks,
                            false,
                        );
                    }
                    KeyCode::Char('u' | 'U') => {
                        self.local_stats.sort(
                            self.index.clone(),
                            data::SortingMode::LocalOutLinks,
                            false,
                        );
                    }
                    KeyCode::Char('i' | 'I') => {
                        self.local_stats.sort(
                            self.index.clone(),
                            data::SortingMode::GlobalInLinks,
                            false,
                        );
                    }
                    KeyCode::Char('n' | 'N') => {
                        self.local_stats.sort(
                            self.index.clone(),
                            data::SortingMode::LocalInLinks,
                            false,
                        );
                    }
                    KeyCode::Char('b' | 'B') => {
                        self.local_stats
                            .sort(self.index.clone(), data::SortingMode::Broken, false);
                    }
                    KeyCode::Char('m' | 'M') => {
                        self.local_stats.sort(
                            self.index.clone(),
                            data::SortingMode::LastModified,
                            false,
                        );
                    }
                    KeyCode::Char('r' | 'R') => {
                        self.local_stats.reverse_order();
                    }
                    KeyCode::Esc => {
                        self.selected = old_selected;
                    }
                    _ => {}
                }
            }
        };

        Ok(ui::Message::None)
    }

    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer) {
        // Get the filter string (neccssary to determine if a filter is active)
        let (global_size, local_size) = self.stats_heights(self.filter_area.lines().last());
        // Vertical layout
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(global_size),
            Constraint::Length(local_size),
            Constraint::Length(3),
            Constraint::Min(6),
        ]);

        // Generate areas
        let [title_area, global_stats_area, local_stats_area, filter_area, table_area] =
            vertical.areas(area);

        // Title
        let title = Line::from(vec![Span::styled(
            self.manager.get_vault_title(),
            self.styles.title_style,
        )])
        .alignment(Alignment::Center);

        let version = Line::from(vec![Span::styled(
            format!("rucola v{}", env!("CARGO_PKG_VERSION")),
            self.styles.subtitle_style,
        )])
        .alignment(Alignment::Right);

        // Generate stats areas
        let global_stats =
            self.global_stats
                .to_global_stats_table(&self.styles)
                .block(Block::bordered().title(style::Styled::set_style(
                    "Global Statistics",
                    self.styles.title_style,
                )));

        let local_stats = self
            .local_stats
            .to_local_stats_table(&self.global_stats, &self.styles)
            .block(Block::bordered().title(style::Styled::set_style(
                "Local Statistics",
                self.styles.title_style,
            )));

        // === Table Area ===

        // Generate state from selected element
        let mut state = TableState::new()
            .with_offset(
                self.selected
                    // try to keep element at above 1/3rd of the total height
                    .saturating_sub(table_area.height as usize / 3)
                    .min(
                        // but when reaching the end of the list, still scroll down
                        self.local_stats
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
                | SelectMode::Delete
                | SelectMode::SubmenuFile
                | SelectMode::SubmenuSorting => Some(self.selected),
                SelectMode::Filter | SelectMode::FilterHelp | SelectMode::Create => None,
            });

        // Instructions at the bottom of the page
        let instructions_bot_left = Line::from(vec![
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
        ])
        .left_aligned();

        // Display some git info:
        let git_info = if let Some(git_manager) = &self.git_manager {
            let mut info = "Git ".to_owned();
            let (ahead, behind) = git_manager.calculate_ahead_behind().unwrap_or((0, 0));
            let (untracked, uncommited) = git_manager.changes();

            if ahead > 0 {
                info.push('');
            }

            if behind > 0 {
                info.push('');
            }

            if untracked {
                info.push('!');
            }

            if uncommited {
                info.push('+');
            }

            info.push_str("──");

            info
        } else {
            String::new()
        };

        let instructions_bot_right = Line::from(vec![
            Span::styled("E", self.styles.hotkey_style),
            Span::styled("dit──", self.styles.text_style),
            Span::styled("V", self.styles.hotkey_style),
            Span::styled("iew──", self.styles.text_style),
            Span::styled("S", self.styles.hotkey_style),
            Span::styled("orting──", self.styles.text_style),
            Span::styled(git_info, self.styles.text_style),
            Span::styled("M", self.styles.hotkey_style),
            Span::styled("anage Files──", self.styles.text_style),
            Span::styled("Q", self.styles.hotkey_style),
            Span::styled("uit", self.styles.text_style),
        ])
        .right_aligned();

        let table_heading_key_style = if self.mode == SelectMode::SubmenuSorting {
            self.styles.hotkey_style
        } else {
            self.styles.subtitle_style
        };

        // Finally generate the table from the generated row and width data
        let table = self
            .local_stats
            .to_note_table(self.index.clone(), &self.styles)
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
            .row_highlight_style(self.styles.selected_style)
            // Add Instructions and a title
            .block(
                Block::bordered()
                    .title_top(style::Styled::set_style("Notes", self.styles.title_style))
                    .title_bottom(instructions_bot_left)
                    .title_bottom(instructions_bot_right),
            );

        // === Rendering ===
        Widget::render(title, title_area, buf);
        Widget::render(version, title_area, buf);

        Widget::render(&self.filter_area, filter_area, buf);

        Widget::render(global_stats, global_stats_area, buf);
        Widget::render(local_stats, local_stats_area, buf);

        StatefulWidget::render(table, table_area, buf, &mut state);

        // Render possible pop-ups
        match self.mode {
            SelectMode::SubmenuFile | SelectMode::SubmenuSorting => {
                let contents = if self.mode == SelectMode::SubmenuFile {
                    vec![
                        ("N", "New note"),
                        ("R", "Rename selected note"),
                        ("M", "Move selected note"),
                        ("D", "Delete selected note"),
                    ]
                } else {
                    vec![
                        ("S", "Shuffle"),
                        ("A", "Sort by name"),
                        ("W", "Sort by words"),
                        ("C", "Sort by characters"),
                        ("O", "Sort by global outlinks"),
                        ("U", "Sort by local outlinks"),
                        ("I", "Sort by global inlinks"),
                        ("N", "Sort by local inlinks"),
                        ("B", "Sort by broken links"),
                        ("M", "Sort by last modification"),
                        ("R", "Reverse sorting"),
                    ]
                }
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect_vec();

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
                Widget::render(&self.name_area, center_area, buf);
            }
            SelectMode::Delete => {
                let delete_confirmation = Paragraph::new(Text::styled(
                    format!(
                        "Delete selected note \"{}\"?",
                        self.local_stats
                            .get_selected(self.selected)
                            .and_then(|note| self
                                .index
                                .borrow()
                                .get(&note.id)
                                .map(|note| note.display_name.clone()))
                            .unwrap_or(String::from("<Unknown Note>"))
                    ),
                    self.styles.text_style,
                ))
                .block(
                    Block::bordered()
                        .title(style::Styled::set_style(
                            "Confirm deletion",
                            self.styles.title_style,
                        ))
                        .title_bottom(
                            Line::from(vec![
                                Span::styled("󰌑", self.styles.hotkey_style),
                                Span::styled(": Confirm──", self.styles.text_style),
                                Span::styled("Esc", self.styles.hotkey_style),
                                Span::styled("/", self.styles.text_style),
                                Span::styled("Any", self.styles.hotkey_style),
                                Span::styled(": Cancel", self.styles.text_style),
                            ])
                            .right_aligned(),
                        ),
                );

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
                Widget::render(delete_confirmation, center_area, buf);
            }
            SelectMode::FilterHelp => {
                let help_widths = [Constraint::Length(9), Constraint::Min(0)];

                let help_rows = [
                    Row::new(vec![
                        Cell::from("/ or F").style(self.styles.subtitle_style),
                        Cell::from("Enter the filter text box.").style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from("󰌑 or Esc").style(self.styles.subtitle_style),
                        Cell::from("Exit the filter text box").style(self.styles.text_style),
                    ]),
                    Row::new(vec![Cell::from("").style(self.styles.subtitle_style)]),
                    Row::new(vec![
                        Cell::from("#[tag]").style(self.styles.subtitle_style),
                        Cell::from("Show notes with tag [tag].").style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from("!#[tag]").style(self.styles.subtitle_style),
                        Cell::from("Show notes without tag [tag].").style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from("").style(self.styles.subtitle_style),
                        Cell::from("Use '-' instead of ' ' for multi-word tags.")
                            .style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from(" ").style(self.styles.subtitle_style),
                        Cell::from("").style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from(">[note]").style(self.styles.subtitle_style),
                        Cell::from("Show notes linking to [note].").style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from("<[note]").style(self.styles.subtitle_style),
                        Cell::from("Show notes linked to from [note].")
                            .style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from("!>[note]").style(self.styles.subtitle_style),
                        Cell::from("Show notes not linking to [note].")
                            .style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from("!<[note]").style(self.styles.subtitle_style),
                        Cell::from("Show notes not linked to from [note].")
                            .style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from("").style(self.styles.subtitle_style),
                        Cell::from("Use '-' instead of ' ' for multi-word titles.")
                            .style(self.styles.text_style),
                    ]),
                    Row::new(vec![Cell::from("").style(self.styles.subtitle_style)]),
                    Row::new(vec![
                        Cell::from("|").style(self.styles.subtitle_style),
                        Cell::from("All text after | will be searched in the full text.")
                            .style(self.styles.text_style),
                    ]),
                    Row::new(vec![
                        Cell::from(" ").style(self.styles.subtitle_style),
                        Cell::from("All other text will be matched against the title.")
                            .style(self.styles.text_style),
                    ]),
                ];

                // Pop-up should be as tall as the number of help dialog rows
                // plus 2 rows for top and bottom border
                let popup_height = help_rows.len() as u16 + 2;
                let help_table = Table::new(help_rows, help_widths).column_spacing(1).block(
                    Block::bordered()
                        .title(style::Styled::set_style(
                            "Filter Syntax",
                            self.styles.title_style,
                        ))
                        .title_bottom(
                            Line::from(vec![
                                Span::styled("C", self.styles.hotkey_style),
                                Span::styled("lose", self.styles.text_style),
                            ])
                            .right_aligned(),
                        ),
                );

                let popup_areas = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(popup_height),
                    Constraint::Fill(1),
                ])
                .split(area);

                let center_area = Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Length(64),
                    Constraint::Fill(1),
                ])
                .split(popup_areas[1])[1];

                // Clear the area and then render the help menu on top.
                Widget::render(Clear, center_area, buf);
                Widget::render(help_table, center_area, buf);
            }
        }
    }
}
