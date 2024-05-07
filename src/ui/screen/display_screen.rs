use std::{collections::HashMap, rc::Rc};

use crate::{config, data, ui};

use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    /// The internal stats of the displayed note.
    note: data::Note,
    /// The used config
    config: config::Config,
    /// Array of all the link tables, in the order
    /// - backlinks
    /// - links
    /// - l2 backlinks
    /// - l2 links
    links: [Vec<(String, String)>; 4],
    /// The index of the note selected in each table
    selected: [usize; 4],
    /// The index of the primary table currently focused
    foc_table: usize,
}

impl DisplayScreen {
    /// Creates a new display screen for the specified note, remembering relevant parts of the config.
    pub fn new(
        note_id: String,
        index: Rc<HashMap<String, data::Note>>,
        config: &config::Config,
    ) -> Option<Self> {
        // Cache the note
        let note = index.get(&note_id).cloned()?;

        // Get level 1 links
        let l1links = note
            .links
            .iter()
            .flat_map(|id| {
                index
                    .get(id)
                    .map(|note| note.name.clone())
                    .map(|name| (id.to_owned(), name))
            })
            // remove duplicates
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect_vec();

        // Get level 2 links
        let l2links = l1links
            .iter()
            .filter_map(|(id, _name)| index.get(id).map(|note| &note.links))
            .flatten()
            .flat_map(|id| {
                index
                    .get(id)
                    .map(|note| note.name.clone())
                    .map(|name| (id.to_owned(), name))
            })
            // remove duplicates
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect_vec();

        // Get level 1 backlinks
        let l1blinks = index
            .iter()
            .filter(|(_key, value)| value.links.contains(&note_id))
            .map(|(key, value)| (key.to_owned(), value.name.to_owned()))
            .collect_vec();

        let l2blinks = index
            .iter()
            .filter(|(_key, value)| l1blinks.iter().any(|(id, _name)| value.links.contains(id)))
            .map(|(key, value)| (key.to_owned(), value.name.to_owned()))
            .collect_vec();

        Some(Self {
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

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<ui::Message> {
        match key.code {
            // Quit with q
            KeyCode::Char('Q' | 'q') => Some(ui::Message::Quit),
            // Go back to selection with f
            KeyCode::Char('F' | 'f') => Some(ui::Message::SwitchSelect),
            // Go up in the current list with k
            KeyCode::Up | KeyCode::Char('K' | 'k') => {
                self.selected
                    .get_mut(self.foc_table)
                    .map(|selected| *selected = selected.saturating_sub(1));
                None
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
                None
            }
            // Change list with Tab or L
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('L' | 'l') => {
                self.foc_table = (self.foc_table.wrapping_add(1)) % 4;
                None
            }
            // Change list back with Shift+Tab or H
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('H' | 'h') => {
                self.foc_table = (self.foc_table.wrapping_sub(1)) % 4;
                None
            }
            // If enter, switch to that note
            KeyCode::Enter => {
                self.links
                    // get the correct table
                    .get(self.foc_table)
                    // unwrap the current index
                    .and_then(|table| table.get(self.selected[self.foc_table]))
                    // and extract the id
                    .map(|(id, _name)| ui::Message::SwitchDisplay(id.to_owned()))
            }
            // Open selected item in editor
            KeyCode::Char('e' | 'E') => {
                let path = std::path::Path::new(&self.note.path);
                Some(ui::Message::OpenExternalCommand(
                    // check if there is an application configured
                    if let Some(application) = self.config.get_editor() {
                        // default configures -> create a command for that one
                        open::with_command(path, application)
                    } else {
                        // else -> get system defaults, take the first one
                        open::commands(path).remove(0)
                    },
                ))
            }
            _ => None,
        }
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

        // Table
        let table = Table::new(rows, [Constraint::Min(20)])
            .highlight_style(if index == self.foc_table {
                styles.selected_style
            } else {
                styles.text_style
            })
            .block(Block::bordered().title(title).title(count));

        StatefulWidget::render(table, area, buf, &mut state);
    }
}
