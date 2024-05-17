use crate::{config, data, error, ui};

use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    // === CONFIG ===
    /// The used config
    config: config::Config,

    // === DATA ===
    /// The internal stats of the displayed note.
    note: data::Note,
    /// Array of all the link tables, in the order
    /// - backlinks
    /// - links
    /// - l2 backlinks
    /// - l2 links
    links: [Vec<(String, String)>; 4],

    // === UI ===
    /// The index of the note selected in each table
    selected: [usize; 4],
    /// The index of the primary table currently focused
    foc_table: usize,
}

impl DisplayScreen {
    /// Creates a new display screen for the specified note, remembering relevant parts of the config.
    pub fn new(
        note_id: &str,
        index: data::NoteIndexContainer,
        config: &config::Config,
    ) -> Result<Self, error::RucolaError> {
        let index = index.borrow();
        // Cache the note
        let note = index
            .get(note_id)
            .ok_or_else(|| error::RucolaError::NoteNoteFound(note_id.to_owned()))
            .cloned()?;

        // Get level 1 links
        let l1links = index.links_vec(note_id);

        // Get level 2 links
        let l2links = l1links
            .iter()
            .flat_map(|(id, _name)| index.links_vec(id))
            .collect_vec();

        // Get level 1 backlinks
        let l1blinks = index.blinks_vec(note_id);
        // Get level 2 backlinks
        let l2blinks = l1blinks
            .iter()
            .flat_map(|(id, _name)| index.blinks_vec(id))
            .collect();

        Ok(Self {
            config: config.clone(),
            links: [l1blinks, l1links, l2blinks, l2links],
            note,
            selected: [0; 4],
            foc_table: 0,
        })
    }
}

impl super::Screen for DisplayScreen {
    fn draw(
        &self,
        area: ratatui::prelude::layout::Rect,
        buf: &mut ratatui::prelude::buffer::Buffer,
    ) {
        // Cache style
        let styles = self.config.get_ui_styles();
        // Generate vertical layout
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);

        let [title_area, stats_area, links1_area, links2_area] = vertical.areas(area);

        // Title
        let title = Line::from(vec![Span::styled(
            self.note.name.as_str(),
            styles.title_style,
        )])
        .alignment(Alignment::Center);

        // Display the note's tags
        let tags = self
            .note
            .tags
            .iter()
            .enumerate()
            .map(|(index, s)| {
                [
                    Span::styled(if index == 0 { "" } else { ", " }, styles.text_style),
                    Span::styled(s.as_str(), styles.subtitle_style),
                ]
            })
            .flatten()
            .collect_vec();

        // Stats Area
        let stats_rows = [
            Row::new(vec![
                Cell::from("Words:"),
                Cell::from(format!("{:7}", self.note.words)),
                Cell::from("Tags:"),
                Cell::from(Line::from(tags)),
            ]),
            Row::new(vec![
                Cell::from("Chars:"),
                Cell::from(format!("{:7}", self.note.characters)),
                Cell::from("Path:"),
                Cell::from(self.note.path.to_str().unwrap_or_default()),
            ]),
        ];

        let stats_widths = [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Min(20),
        ];

        let instructions_top = block::Title::from(Line::from(vec![
            Span::styled("E", styles.hotkey_style),
            Span::styled("dit Note", styles.text_style),
        ]))
        .alignment(Alignment::Right)
        .position(block::Position::Top);

        let stats = Table::new(stats_rows, stats_widths)
            .column_spacing(1)
            .block(
                Block::bordered()
                    .title("Statistics".set_style(styles.title_style))
                    .title(instructions_top),
            );

        // === All the links ===

        let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]);

        let [blinks1, links1] = horizontal.areas(links1_area);
        let [blinks2, links2] = horizontal.areas(links2_area);

        Widget::render(title, title_area, buf);
        Widget::render(stats, stats_area, buf);

        self.draw_link_table(0, "Backlinks", blinks1, buf);
        self.draw_link_table(1, "Links", links1, buf);
        self.draw_link_table(2, "Level 2 Backlinks", blinks2, buf);
        self.draw_link_table(3, "Level 2 Links", links2, buf);
    }

    fn update(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<ui::Message, error::RucolaError> {
        Ok(match key.code {
            // Quit with Q
            KeyCode::Char('Q' | 'q') => ui::Message::Quit,
            // Go back to selection with f
            KeyCode::Char('F' | 'f') => ui::Message::DisplayStackClear,
            // Return to selection or previous note with left or H
            KeyCode::Left | KeyCode::Char('H' | 'h') => ui::Message::DisplayStackPop,
            // Go up in the current list with k
            KeyCode::Up | KeyCode::Char('K' | 'k') => {
                self.selected
                    .get_mut(self.foc_table)
                    .map(|selected| *selected = selected.saturating_sub(1));
                ui::Message::None
            }
            // Go down in the current list with j
            KeyCode::Down | KeyCode::Char('J' | 'j') => {
                self.selected.get_mut(self.foc_table).map(|selected| {
                    *selected = selected.saturating_add(1).min(
                        self.links
                            .get(self.foc_table)
                            .map(|list| list.len().saturating_sub(1))
                            .unwrap_or_default(),
                    )
                });
                ui::Message::None
            }
            // Change list with Tab
            KeyCode::Tab => {
                self.foc_table = (self.foc_table.wrapping_add(1)) % 4;
                ui::Message::None
            }
            // Change list back with Shift+Tab or H
            KeyCode::BackTab => {
                self.foc_table = (self.foc_table.wrapping_sub(1)) % 4;
                ui::Message::None
            }
            // If enter, switch to that note
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('L' | 'l') => {
                self.links
                    // get the correct table
                    .get(self.foc_table)
                    // unwrap the current index
                    .and_then(|table| table.get(self.selected[self.foc_table]))
                    // and extract the id
                    .map(|(id, _name)| ui::Message::DisplayStackPush(id.to_owned()))
                    .unwrap_or(ui::Message::None)
            }
            // Open selected item in editor
            KeyCode::Char('e' | 'E') => ui::Message::OpenExternalCommand(
                self.config.create_opening_command(&self.note.path)?,
            ),

            _ => ui::Message::None,
        })
    }
}

impl DisplayScreen {
    fn draw_link_table(&self, index: usize, title: &str, area: Rect, buf: &mut Buffer) {
        // Cache style
        let styles = self.config.get_ui_styles();
        // Title
        let title = block::Title::from(Line::from(vec![Span::styled(title, styles.title_style)]))
            .alignment(Alignment::Left)
            .position(block::Position::Top);

        let count = self
            .links
            .get(index)
            .map(|list| list.len())
            .unwrap_or_default();

        // Count
        let count = block::Title::from(Line::from(vec![Span::styled(
            format!("{} Note{}", count, if count == 1 { "" } else { "s" }),
            styles.text_style,
        )]))
        .alignment(Alignment::Right)
        .position(block::Position::Top);

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
                    .map(|(_id, name)| Row::new(vec![Span::from(name)]))
                    .collect_vec()
            })
            .unwrap_or_default();

        // create default surrounding block
        let block = Block::bordered().title(title).title(count);

        // in some places, add instructions
        let block = match index {
            2 => block.title(
                block::Title::from(Line::from(vec![
                    Span::styled("J", styles.hotkey_style),
                    Span::styled("/", styles.text_style),
                    Span::styled("", styles.hotkey_style),
                    Span::styled(": Down──", styles.text_style),
                    Span::styled("K", styles.hotkey_style),
                    Span::styled("/", styles.text_style),
                    Span::styled("", styles.hotkey_style),
                    Span::styled(": Up──", styles.text_style),
                    Span::styled("Enter", styles.hotkey_style),
                    Span::styled("/", styles.text_style),
                    Span::styled("L", styles.hotkey_style),
                    Span::styled("/", styles.text_style),
                    Span::styled("", styles.hotkey_style),
                    Span::styled(": Open──", styles.text_style),
                    Span::styled("H", styles.hotkey_style),
                    Span::styled("/", styles.text_style),
                    Span::styled("", styles.hotkey_style),
                    Span::styled(": Back", styles.text_style),
                ]))
                .alignment(Alignment::Left)
                .position(block::Position::Bottom),
            ),
            3 => block.title(
                block::Title::from(Line::from(vec![
                    Span::styled("Tab", styles.hotkey_style),
                    Span::styled(": Next Table──", styles.text_style),
                    Span::styled("Shift+Tab", styles.hotkey_style),
                    Span::styled(": Previous Table", styles.text_style),
                ]))
                .alignment(Alignment::Right)
                .position(block::Position::Bottom),
            ),
            _ => block,
        };

        // Table
        let table = Table::new(rows, [Constraint::Min(20)])
            .highlight_style(if index == self.foc_table {
                styles.selected_style
            } else {
                styles.text_style
            })
            .block(block);

        StatefulWidget::render(table, area, buf, &mut state);
    }
}
