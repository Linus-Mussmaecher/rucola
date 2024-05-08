use crate::{config, data, ui};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

use std::io::Write;
use std::rc::Rc;
use tui_textarea::TextArea;

/// Describes the current mode of the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SelectMode {
    /// Selecting a note from the list.
    #[default]
    Select,
    /// Typing into the filter box.
    Filter,
    /// Typing into the create box.
    Create,
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
    // === DATA ===
    /// A reference to the index of all notes
    index: Rc<data::NoteIndex>,
    /// The currently displayed statistics for all notes.
    local_stats: data::EnvironmentStats,
    /// The currently displayed statistics for all notes matching the current filter.
    global_stats: data::EnvironmentStats,

    // === CONFIG ===
    /// The config used.
    config: config::Config,

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
    pub fn new(index: Rc<data::NoteIndex>, config: &config::Config) -> Self {
        let mut res = Self {
            local_stats: data::EnvironmentStats::new_with_filters(&index, data::Filter::default()),
            global_stats: data::EnvironmentStats::new_with_filters(&index, data::Filter::default()),
            index,
            filter_area: TextArea::default(),
            create_area: TextArea::default(),
            mode: SelectMode::Select,
            config: config.clone(),
            all_tags: false,
            sorting: SortingMode::Name,
            sorting_asc: true,
            selected: 0,
            dynamic_filter: config.get_dynamic_filter(),
        };

        res.sort();

        res.style_text_area();

        res
    }

    /// Styling of TextArea extracted from constructor to keep it clean.
    fn style_text_area(&mut self) {
        let styles = self.config.get_ui_styles();

        // === Filter ===

        // The actual title
        let title_top = block::Title::from(Line::from(vec![
            Span::styled("F", styles.hotkey_style),
            Span::styled("ilter", styles.title_style),
        ]));

        // The hotkey instructions at the bottom.
        let instructions = block::Title::from(Line::from(vec![
            Span::styled("C", styles.hotkey_style),
            Span::styled("lear filter", styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Top);

        let instructions_bot = block::Title::from(Line::from(vec![
            Span::styled(
                if self.all_tags { "All " } else { "Any " },
                styles.text_style,
            ),
            Span::styled("T", styles.hotkey_style),
            Span::styled("ags", styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Bottom);

        // Apply default styles to the filter area

        self.filter_area.set_style(styles.input_style);
        self.filter_area.set_cursor_line_style(styles.input_style);

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
            styles.title_style,
        )]));

        // Apply default styles to the create area

        self.create_area.set_style(styles.input_style);
        self.create_area.set_cursor_line_style(styles.input_style);

        self.create_area
            .set_block(Block::bordered().title(title_top));
    }

    /// Creates a filter from the current content of the text area.
    fn filter_from_input(&self) -> data::Filter {
        // We should only have one line, read that one
        if let Some(line) = self.filter_area.lines().first() {
            let mut filter = data::Filter::default();
            // default filter is this line with all white space removed
            filter.title = line.chars().filter(|c| !c.is_whitespace()).collect();

            // Go through words
            for word in line.split_whitespace() {
                if word.starts_with('#') {
                    // words with a hash count as a tag
                    filter.tags.push(word.to_string());
                    // and remove it from the title to match
                    filter.title = filter.title.replace(word, "");
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
    fn update(&mut self, key: KeyEvent) -> Option<crate::ui::Message> {
        // Check for mode
        match self.mode {
            // Main mode: Switch to modes, general command
            SelectMode::Select => match key.code {
                // F: Go to filter mode
                KeyCode::Char('f' | 'F') => {
                    self.mode = SelectMode::Filter;
                }
                // N: Go to create mode
                KeyCode::Char('n' | 'N') => {
                    self.mode = SelectMode::Create;
                }
                // R: Refresh index
                KeyCode::Char('r' | 'R') => {
                    return Some(ui::Message::SwitchSelect { refresh: true })
                }
                // C: Clear filter
                KeyCode::Char('c' | 'C') => {
                    self.filter_area.select_all();
                    self.filter_area.cut();
                    self.filter(data::Filter::default());
                }
                // Q: Quit application
                KeyCode::Char('q' | 'Q') => return Some(crate::ui::Message::Quit),
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
                KeyCode::Char('m' | 'M') => {
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
                KeyCode::Char('g' | 'G') => {
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
                KeyCode::Enter => {
                    if let Some(env_stats) = self.local_stats.filtered_stats.get(self.selected) {
                        return Some(crate::ui::Message::SwitchDisplay(env_stats.id.clone()));
                    }
                }
                // Open selected item in editor
                KeyCode::Char('e' | 'E') => {
                    return self
                        // get the selected item in the list for the id
                        .local_stats
                        .filtered_stats
                        .get(self.selected)
                        // use this id in the index to get the note
                        .and_then(|env_stats| self.index.get(&env_stats.id))
                        // get the path from the note
                        .map(|note| {
                            let path = std::path::Path::new(&note.path);
                            ui::Message::OpenExternalCommand(
                                // check if there is an application configured
                                if let Some(application) = self.config.get_editor() {
                                    // default configures -> create a command for that one
                                    open::with_command(path, application)
                                } else {
                                    // else -> get system defaults, take the first one
                                    open::commands(path).remove(0)
                                },
                            )
                        });
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
                        self.filter_area.input_without_shortcuts(key);
                        if self.dynamic_filter {
                            self.filter(self.filter_from_input());
                        }
                    }
                };
            }
            // Create mode: Type in note name
            SelectMode::Create => {
                match key.code {
                    // Escape: Back to main mode, clear the buffer
                    KeyCode::Esc => {
                        self.filter_area.select_all();
                        self.filter_area.cut();
                        self.mode = SelectMode::Select;
                    }
                    // Enter: Create note, back to main mode, clear teh buffer
                    KeyCode::Enter => {
                        // Piece together the file path
                        let mut path = self.config.get_vault_path();
                        path.push(
                            self.create_area
                                .lines()
                                .first()
                                .map(|s| s.as_str().trim_start_matches("/"))
                                .unwrap_or("Untitled"),
                        );
                        path.set_extension("md");
                        // Create the file
                        let file = std::fs::File::create(path);
                        if let Ok(mut file) = file {
                            let _ = write!(
                                file,
                                "#{}",
                                self.create_area
                                    .lines()
                                    .first()
                                    .map(|s| s.as_str())
                                    .unwrap_or("Untitled")
                            );
                        }
                        // Clear the input area
                        self.create_area.select_all();
                        self.create_area.cut();
                        // Switch back to base mode
                        self.mode = SelectMode::Select;
                    }
                    // All other key events are passed on to the text area, then the filter is immediately applied
                    _ => {
                        // Else -> Pass on to the text area
                        self.create_area.input_without_shortcuts(key);
                    }
                };
            }
        }

        None
    }

    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer) {
        // Cache styles
        let styles = self.config.get_ui_styles();

        // Vertical layout
        let vertical = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(6),
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
            .block(Block::bordered().title("Global Statistics".set_style(styles.title_style)));

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
            .block(Block::bordered().title("Local Statistics".set_style(styles.title_style)));

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
            // If not in filter mode, show a selected element
            .with_selected(match self.mode {
                SelectMode::Select => Some(self.selected),
                SelectMode::Filter | SelectMode::Create => None,
            });

        // Generate row data
        let notes_rows = self
            .local_stats
            .filtered_stats
            .iter()
            .flat_map(|env_stats| {
                // generate the stats row for each element
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
            .collect::<Vec<Row>>();

        // Instructions at the bottom of the page
        let instructions_bot = block::Title::from(Line::from(vec![
            Span::styled("N", styles.hotkey_style),
            Span::styled("ew Note──", styles.text_style),
            Span::styled("E", styles.hotkey_style),
            Span::styled("dit Note──", styles.text_style),
            Span::styled(
                if self.sorting_asc {
                    "Ascending "
                } else {
                    "Descending "
                },
                styles.text_style,
            ),
            Span::styled("S", styles.hotkey_style),
            Span::styled("orting", styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Bottom);

        // Finally generate the table from the generated row and width data
        let table = Table::new(notes_rows, notes_table_widths)
            .column_spacing(1)
            // Add Headers
            .header(Row::new(vec![
                Line::from(vec![
                    Span::styled("Na", styles.title_style),
                    Span::styled("m", styles.hotkey_style),
                    Span::styled("e", styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("W", styles.hotkey_style),
                    Span::styled("ords", styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("Ch", styles.title_style),
                    Span::styled("a", styles.hotkey_style),
                    Span::styled("rs", styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("Global", styles.title_style),
                    Span::styled("O", styles.hotkey_style),
                    Span::styled("ut", styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("LocalO", styles.title_style),
                    Span::styled("u", styles.hotkey_style),
                    Span::styled("t", styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("G", styles.hotkey_style),
                    Span::styled("lobalIn", styles.title_style),
                ]),
                Line::from(vec![
                    Span::styled("L", styles.hotkey_style),
                    Span::styled("ocalIn", styles.title_style),
                ]),
            ]))
            .highlight_style(styles.selected_style)
            // Add Instructions and a title
            .block(
                Block::bordered()
                    .title("Notes".set_style(styles.title_style))
                    .title(instructions_bot),
            );

        // === Create pop-up

        // Mostly styled on creation
        let create_input = self.create_area.widget();

        // === Rendering ===

        Widget::render(filter_input, filter_area, buf);

        Widget::render(global_stats, global_stats_area, buf);
        Widget::render(local_stats, local_stats_area, buf);

        StatefulWidget::render(table, table_area, buf, &mut state);

        // Render the pop up
        if self.mode == SelectMode::Create {
            let popup_layout = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .split(area);

            let create_area = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Percentage(60),
                Constraint::Fill(1),
            ])
            .split(popup_layout[1])[1];

            Widget::render(Clear, create_area, buf);
            Widget::render(create_input, create_area, buf);
        }
    }
}
