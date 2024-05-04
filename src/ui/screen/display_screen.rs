use std::{collections::HashMap, rc::Rc};

use crate::{config, data, ui};

use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    /// The internal stats of the displayed note.
    note: data::Note,
    /// The used styling theme
    styles: ui::UiStyles,
    /// Links from the note
    l1links: Vec<(String, String)>,
    /// Backlinks from the note
    l1blinks: Vec<(String, String)>,
    /// Links from links from the note
    l2links: Vec<(String, String)>,
    /// Backlinks from backlinks from the note
    l2blinks: Vec<(String, String)>,
}

impl DisplayScreen {
    /// Creates a new display screen for the specified note, remembering relevant parts of the config.
    pub fn new(
        note_id: String,
        index: Rc<HashMap<String, data::Note>>,
        config: &config::Config,
    ) -> color_eyre::Result<Self> {
        // Cache the note
        let note = index
            .get(&note_id)
            .cloned()
            .ok_or(eyre::eyre!("No such note."))?;

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

        Ok(Self {
            styles: config.get_ui_styles().to_owned(),
            l1links,
            l1blinks,
            l2links,
            l2blinks,
            note,
        })
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
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);

        let [title_area, stats_area, links1_area, links2_area] = vertical.areas(area);

        // Title
        let title = Line::from(vec![Span::styled(
            self.note.name.as_str(),
            self.styles.title_style,
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
                    Span::styled(if index == 0 { "" } else { ", " }, self.styles.text_style),
                    Span::styled(s.as_str(), self.styles.subtitle_style),
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

        let stats = Table::new(stats_rows, stats_widths)
            .column_spacing(1)
            .block(Block::bordered().title("Statistics".set_style(self.styles.title_style)));

        // === All the links ===

        let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]);

        let [blinks1, links1] = horizontal.areas(links1_area);
        let [blinks2, links2] = horizontal.areas(links2_area);

        Widget::render(title, title_area, buf);
        Widget::render(stats, stats_area, buf);

        self.draw_link_table("Links", &self.l1links, links1, buf);
        self.draw_link_table("Backlinks", &self.l1blinks, blinks1, buf);
        self.draw_link_table("Level 2 Links", &self.l2links, links2, buf);
        self.draw_link_table("Level 2 Backlinks", &self.l2blinks, blinks2, buf);
    }

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<ui::Message> {
        match key.code {
            crossterm::event::KeyCode::Char('Q' | 'q') => Some(ui::Message::Quit),
            crossterm::event::KeyCode::Char('F' | 'f') => Some(ui::Message::SwitchSelect),

            _ => None,
        }
    }
}

impl DisplayScreen {
    fn draw_link_table(
        &self,
        title: &str,
        link_list: &[(String, String)],
        area: Rect,
        buf: &mut Buffer,
    ) {
        // Title
        let title = block::Title::from(Line::from(vec![Span::styled(
            title,
            self.styles.title_style,
        )]))
        .alignment(Alignment::Left)
        .position(block::Position::Top);

        // Count
        let count = block::Title::from(Line::from(vec![Span::styled(
            format!("{} Notes", link_list.len()),
            self.styles.text_style,
        )]))
        .alignment(Alignment::Right)
        .position(block::Position::Top);

        // Instructions

        // Rows
        let rows = link_list
            .iter()
            .map(|(_id, name)| Row::new(vec![Span::from(name)]))
            .collect_vec();

        // Table
        let table = Table::new(rows, [Constraint::Min(20)])
            .highlight_style(self.styles.selected_style)
            .block(Block::bordered().title(title).title(count));

        Widget::render(table, area, buf);
    }
}
