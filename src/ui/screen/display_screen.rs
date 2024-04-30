use std::{collections::HashMap, rc::Rc};

use crate::{config, data, ui};

use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    /// A reference to the index of all notes
    index: Rc<HashMap<String, data::Note>>,
    /// The internal stats of the displayed note.
    note: data::Note,
    /// The used styling theme
    styles: ui::UiStyles,
}

impl DisplayScreen {
    /// Creates a new display screen for the specified note, remembering relevant parts of the config.
    pub fn new(
        note_id: String,
        index: Rc<HashMap<String, data::Note>>,
        config: &config::Config,
    ) -> color_eyre::Result<Self> {
        Ok(Self {
            note: index.get(&note_id).cloned().unwrap_or_default(),
            index,
            styles: config.get_ui_styles().to_owned(),
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

        Widget::render(title, title_area, buf);
        Widget::render(stats, stats_area, buf);
    }

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<ui::Message> {
        match key.code {
            crossterm::event::KeyCode::Char('Q' | 'q') => Some(ui::Message::Quit),
            crossterm::event::KeyCode::Char('F' | 'f') => Some(ui::Message::SwitchSelect),

            _ => None,
        }
    }
}
