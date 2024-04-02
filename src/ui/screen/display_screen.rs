use std::io::Read;

use crate::{config, data, parser, ui};

use ratatui::{prelude::*, widgets::*};

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    /// User-configured styles to use when displaying markdown.
    styles: ui::MdStyles,
    /// A vector of the parsed markdown tokens to display every frame.
    tokens: Vec<parser::MdBlock>,
    /// The title of the displayed note.
    title: String,
}

impl DisplayScreen {
    pub fn new(note: &data::Note, config: &config::Config) -> color_eyre::Result<Self> {
        let mut file = std::fs::File::open(&note.path)?;

        let mut content = String::new();

        let _ = file.read_to_string(&mut content)?;

        Ok(Self {
            styles: config.get_md_styles().to_owned(),
            tokens: parser::parse_note(&content),
            title: note.name.clone(),
        })
    }
}

impl super::Screen for DisplayScreen {
    fn draw(
        &self,
        area: ratatui::prelude::layout::Rect,
        buf: &mut ratatui::prelude::buffer::Buffer,
    ) {
        let lines = vec![];

        // Create a paragraph from this vec and display it.
        Widget::render(Paragraph::new(lines).wrap(Wrap { trim: true }), area, buf);
    }

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<ui::Message> {
        match key.code {
            crossterm::event::KeyCode::Char('Q' | 'q') => Some(ui::Message::Quit),
            crossterm::event::KeyCode::Char('F' | 'f') => Some(ui::Message::SwitchSelect),

            _ => None,
        }
    }
}
