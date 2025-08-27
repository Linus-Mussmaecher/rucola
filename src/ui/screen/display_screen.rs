use crate::{data, error, io, ui};

use itertools::Itertools;
use ratatui::crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};

/// Describes the current mode of the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum DisplayMode {
    /// Selecting a note from the list.
    #[default]
    Display,
    /// Typing into the create box to rename a note.
    Rename,
    /// Typing into the create box to move a note.
    Move,
    /// Confirming delete
    Delete,
}

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    // === CONFIG ===
    /// The file manager this screen uses to enact the user's file system requests on the file system.
    manager: io::FileManager,
    /// The HtmlBuider this screen uses to continuously build html files.
    builder: io::HtmlBuilder,
    /// The used styles.
    styles: ui::UiStyles,

    // === DATA ===
    /// The internal stats of the displayed note.
    note: data::Note,
    /// A reference to the index of all notes
    index: data::NoteIndexContainer,
    /// Array of all the link tables, in the order
    /// - backlinks
    /// - links
    /// - l2 backlinks
    /// - l2 links
    links: [Vec<(String, String)>; 4],

    // === UI ===
    /// The text area used to create new notes.
    name_area: tui_textarea::TextArea<'static>,
    /// The index of the note selected in each table
    selected: [usize; 4],
    /// The index of the primary table currently focused
    foc_table: usize,
    /// Current input mode
    mode: DisplayMode,
}

impl DisplayScreen {
    /// Creates a new display screen for the specified note, remembering relevant parts of the config.
    pub fn new(
        note_id: &str,
        index: data::NoteIndexContainer,
        manager: io::FileManager,
        builder: io::HtmlBuilder,
        styles: ui::UiStyles,
    ) -> error::Result<Self> {
        let index_b = index.borrow();
        // Cache the note
        let note = index_b
            .get(note_id)
            .ok_or_else(|| error::RucolaError::NoteNotFound(note_id.to_owned()))
            .cloned()?;

        // Get level 1 links
        let l1links = index_b.links_vec(note_id);

        // Get level 2 links
        let l2links = l1links
            .iter()
            .flat_map(|(id, _name)| index_b.links_vec(id))
            .unique()
            .collect_vec();

        // Get level 1 backlinks
        let l1blinks = index_b.blinks_vec(note_id);
        // Get level 2 backlinks
        let l2blinks = l1blinks
            .iter()
            .flat_map(|(id, _name)| index_b.blinks_vec(id))
            .unique()
            .collect();

        // Create input area and style it

        let mut name_area = tui_textarea::TextArea::default();

        name_area.set_style(styles.input_style);
        name_area.set_cursor_line_style(styles.input_style);

        let title_top = block::Title::from(Line::from(vec![Span::styled(
            "Enter note name...",
            styles.title_style,
        )]));
        name_area.set_block(Block::bordered().title(title_top));

        drop(index_b);

        Ok(Self {
            links: [l1blinks, l1links, l2blinks, l2links],
            note,
            index,
            manager,
            builder,
            styles,
            name_area,
            selected: [0; 4],
            foc_table: 0,
            mode: DisplayMode::Display,
        })
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
}

impl super::Screen for DisplayScreen {
    fn draw(
        &self,
        area: ratatui::prelude::layout::Rect,
        buf: &mut ratatui::prelude::buffer::Buffer,
    ) {
        // Generate vertical layout
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);

        let [title_area, stats_area, links1_area, links2_area] = vertical.areas(area);

        // Title
        let title = Line::from(vec![Span::styled(
            self.note.display_name.as_str(),
            self.styles.title_style,
        )])
        .alignment(Alignment::Center);

        let version = Line::from(vec![Span::styled(
            format!("rucola v{}", env!("CARGO_PKG_VERSION")),
            self.styles.subtitle_style,
        )])
        .alignment(Alignment::Right);

        let instructions_bot_right = Line::from(vec![
            Span::styled("V", self.styles.hotkey_style),
            Span::styled("iew──", self.styles.text_style),
            Span::styled("E", self.styles.hotkey_style),
            Span::styled("dit──", self.styles.text_style),
            Span::styled("R", self.styles.hotkey_style),
            Span::styled("ename──", self.styles.text_style),
            Span::styled("M", self.styles.hotkey_style),
            Span::styled("ove──", self.styles.text_style),
            Span::styled("D", self.styles.hotkey_style),
            Span::styled("elete", self.styles.text_style),
        ])
        .right_aligned();

        let stats = self.note.to_stats_table(&self.styles).block(
            Block::bordered()
                .title(style::Styled::set_style(
                    "Statistics",
                    self.styles.title_style,
                ))
                .title_bottom(instructions_bot_right),
        );

        // === All the links ===

        let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]);

        let [blinks1, links1] = horizontal.areas(links1_area);
        let [blinks2, links2] = horizontal.areas(links2_area);

        Widget::render(title, title_area, buf);
        Widget::render(version, title_area, buf);
        Widget::render(stats, stats_area, buf);

        self.draw_link_table(0, "Backlinks", blinks1, buf);
        self.draw_link_table(1, "Links", links1, buf);
        self.draw_link_table(2, "Level 2 Backlinks", blinks2, buf);
        self.draw_link_table(3, "Level 2 Links", links2, buf);

        if self.mode == DisplayMode::Rename
            || self.mode == DisplayMode::Move
            || self.mode == DisplayMode::Delete
        {
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

            if self.mode == DisplayMode::Delete {
                let keys = Line::from(vec![
                    Span::styled("󰌑", self.styles.hotkey_style),
                    Span::styled(": Delete─", self.styles.text_style),
                    Span::styled("Other", self.styles.hotkey_style),
                    Span::styled(": Abort", self.styles.text_style),
                ])
                .centered();

                let del = Paragraph::new(Span::styled(
                    "Are you sure you want to delete?\n",
                    self.styles.text_style,
                ))
                .alignment(Alignment::Center)
                .block(Block::bordered().title_bottom(keys));

                Widget::render(del, center_area, buf);
            } else {
                Widget::render(&self.name_area, center_area, buf);
            }
        }
    }

    fn update(&mut self, key: ratatui::crossterm::event::KeyEvent) -> error::Result<ui::Message> {
        match self.mode {
            DisplayMode::Display => match key.code {
                // Quit with Q
                KeyCode::Char('Q' | 'q') => {
                    return Ok(ui::Message::Quit);
                }
                // Go back to selection with f
                KeyCode::Char('F' | 'f') => {
                    return Ok(ui::Message::DisplayStackClear);
                }
                // Return to selection or previous note with left or H
                KeyCode::Left | KeyCode::Char('H' | 'h') => {
                    return Ok(ui::Message::DisplayStackPop);
                }
                // Go up in the current list with k
                KeyCode::Up | KeyCode::Char('K' | 'k') => {
                    if let Some(selected) = self.selected.get_mut(self.foc_table) {
                        *selected = selected.saturating_sub(1);
                    }
                }
                // Go down in the current list with j
                KeyCode::Down | KeyCode::Char('J' | 'j') => {
                    if let Some(selected) = self.selected.get_mut(self.foc_table) {
                        *selected = selected.saturating_add(1).min(
                            self.links
                                .get(self.foc_table)
                                .map(|list| list.len().saturating_sub(1))
                                .unwrap_or_default(),
                        );
                    }
                }
                // Change list with Tab
                KeyCode::Tab => {
                    self.foc_table = (self.foc_table.wrapping_add(1)) % 4;
                }
                // Change list back with Shift+Tab or H
                KeyCode::BackTab => {
                    self.foc_table = (self.foc_table.wrapping_sub(1)) % 4;
                }
                // If enter, switch to that note
                KeyCode::Enter | KeyCode::Right | KeyCode::Char('L' | 'l') => {
                    return Ok(self
                        .links
                        // get the correct table
                        .get(self.foc_table)
                        // unwrap the current index
                        .and_then(|table| table.get(self.selected[self.foc_table]))
                        // and extract the id
                        .map(|(id, _name)| ui::Message::DisplayStackPush(id.to_owned()))
                        .unwrap_or(ui::Message::None));
                }
                // Open selected item in editor
                KeyCode::Char('e' | 'E') => {
                    return Ok(ui::Message::OpenExternalCommand(Box::new(
                        self.manager.create_edit_command(&self.note.path)?,
                    )));
                }
                // Open selected item in viewer
                KeyCode::Char('v' | 'V') => {
                    self.builder.create_html(&self.note, true)?;
                    return Ok(ui::Message::OpenExternalCommand(Box::new(
                        self.manager
                            .create_view_command(&self.note, key.code == KeyCode::Char('v'))?,
                    )));
                }
                // R: Rename note
                KeyCode::Char('r' | 'R') => {
                    self.mode = DisplayMode::Rename;
                    self.set_name_area("Enter new name of note...", Some(self.note.name.clone()));
                }
                // M: Move note
                KeyCode::Char('m' | 'M') => {
                    self.mode = DisplayMode::Move;
                    self.set_name_area("Enter new location relative to vault...", None);
                }
                // D: Move note
                KeyCode::Char('d' | 'D') => {
                    self.mode = DisplayMode::Delete;
                }

                _ => {}
            },
            DisplayMode::Rename => match key.code {
                KeyCode::Esc => {
                    super::extract_string_and_clear(&mut self.name_area);
                    self.mode = DisplayMode::Display;
                }
                KeyCode::Enter => {
                    self.mode = DisplayMode::Display;
                    self.manager.rename_note_file(
                        self.index.clone(),
                        &data::name_to_id(&self.note.name),
                        super::extract_string_and_clear(&mut self.name_area).ok_or_else(|| {
                            error::RucolaError::Input("New name is empty.".to_string())
                        })?,
                    )?;
                    self.index.borrow().poll_file_system();
                }
                _ => {
                    self.name_area.input(key);
                }
            },
            DisplayMode::Move => match key.code {
                KeyCode::Esc => {
                    super::extract_string_and_clear(&mut self.name_area);
                    self.mode = DisplayMode::Display;
                }
                KeyCode::Enter => {
                    self.mode = DisplayMode::Display;
                    self.manager.move_note_file(
                        self.index.clone(),
                        &data::name_to_id(&self.note.name),
                        super::extract_string_and_clear(&mut self.name_area).ok_or_else(|| {
                            error::RucolaError::Input("Move location is empty.".to_string())
                        })?,
                    )?;
                    self.index.borrow().poll_file_system();
                }

                _ => {
                    self.name_area.input(key);
                }
            },
            DisplayMode::Delete => match key.code {
                KeyCode::Enter => {
                    // delete it from index & filesystem
                    self.manager
                        .delete_note_file(self.index.clone(), &data::name_to_id(&self.note.name))?;
                    self.index.borrow().poll_file_system();
                    return Ok(ui::Message::DisplayStackPop);
                }
                _ => {
                    self.mode = DisplayMode::Display;
                }
            },
        }

        Ok(ui::Message::None)
    }
}

impl DisplayScreen {
    fn draw_link_table(&self, index: usize, title: &str, area: Rect, buf: &mut Buffer) {
        // Title
        let title = Line::from(vec![Span::styled(title, self.styles.title_style)]).left_aligned();

        let count = self
            .links
            .get(index)
            .map(|list| list.len())
            .unwrap_or_default();

        // Count
        let count = Line::from(vec![Span::styled(
            format!("{} Note{}", count, if count == 1 { "" } else { "s" }),
            self.styles.text_style,
        )])
        .right_aligned();

        // Instructions

        // State
        let mut state = self
            .selected
            .get(index)
            .map(|selected_index| {
                TableState::new()
                    .with_offset(
                        selected_index.saturating_sub(area.height as usize / 3).min(
                            // but when reaching the end of the list, still scroll down
                            self.links
                                .get(index)
                                .map(|list| list.len())
                                .unwrap_or(0)
                                // correct for table edges
                                .saturating_add(2)
                                .saturating_sub(area.height as usize),
                        ),
                    )
                    .with_selected(Some(*selected_index))
            })
            .unwrap_or_default();

        *state.offset_mut() = self.selected[index]
            .saturating_sub(area.height as usize / 3)
            .min(
                // but when reaching the end of the list, still scroll down
                self.links
                    .get(index)
                    .map(|list| list.len())
                    .unwrap_or(0)
                    .saturating_sub(area.height as usize)
                    // correct for table edges
                    .saturating_add(2),
            );

        // Rows
        let rows = self
            .links
            .get(index)
            .map(|list| {
                list.iter()
                    .map(|(_id, name)| {
                        Row::new(vec![Span::from(name).style(self.styles.text_style)])
                    })
                    .collect_vec()
            })
            .unwrap_or_default();

        // create default surrounding block
        let block = Block::bordered().title_top(title).title_top(count);

        // in some places, add instructions
        let block = match index {
            2 => block.title_bottom(
                Line::from(vec![
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
                    Span::styled("H", self.styles.hotkey_style),
                    Span::styled("/", self.styles.text_style),
                    Span::styled("", self.styles.hotkey_style),
                    Span::styled(": Back──", self.styles.text_style),
                    Span::styled("F", self.styles.hotkey_style),
                    Span::styled(": Home", self.styles.text_style),
                ])
                .left_aligned(),
            ),
            3 => block.title_bottom(
                Line::from(vec![
                    Span::styled("Tab", self.styles.hotkey_style),
                    Span::styled(": Next Table──", self.styles.text_style),
                    Span::styled("Shift+Tab", self.styles.hotkey_style),
                    Span::styled(": Previous Table", self.styles.text_style),
                ])
                .right_aligned(),
            ),
            _ => block,
        };

        // Table
        let table = Table::new(rows, [Constraint::Min(20)])
            .row_highlight_style(if index == self.foc_table {
                self.styles.selected_style
            } else {
                self.styles.text_style
            })
            .block(block);

        StatefulWidget::render(table, area, buf, &mut state);
    }
}
