use std::io::Read;

use crate::config;
use crate::parser;
use crate::ui;
use ratatui::{prelude::*, widgets::*};

pub struct DisplayScreen {
    content: String,
    styles: ui::Styles,
    paragraphs: Vec<parser::Paragraph>,
}

impl DisplayScreen {
    pub fn new(path: &std::path::Path, config: &config::Config) -> color_eyre::Result<Self> {
        let mut file = std::fs::File::open(path)?;

        let mut content = String::new();

        let _ = file.read_to_string(&mut content)?;

        Ok(Self {
            paragraphs: parser::parse_note(&content),
            content,
            styles: config.get_styles().clone(),
        })
    }
}

impl super::Screen for DisplayScreen {
    fn draw(
        &self,
        area: ratatui::prelude::layout::Rect,
        buf: &mut ratatui::prelude::buffer::Buffer,
    ) {
        let lines = self
            .paragraphs
            .iter()
            .map(|a| a.to_widget())
            .collect::<Vec<_>>();

        Widget::render(
            Paragraph::new(lines)
                .wrap(Wrap { trim: true })
                .style(self.styles.text_style),
            area,
            buf,
        );
    }

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<ui::Message> {
        match key.code {
            crossterm::event::KeyCode::Char('Q' | 'q') => Some(ui::Message::Quit),
            crossterm::event::KeyCode::Char('F' | 'f') => Some(ui::Message::SwitchSelect),

            _ => None,
        }
    }
}
