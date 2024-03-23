use std::io::Read;

use crate::config;
use crate::ui;
use ratatui::{prelude::*, widgets::*};

pub struct DisplayScreen {
    content: String,
    styles: ui::Styles,
}

impl DisplayScreen {
    pub fn new(path: &std::path::Path, config: &config::Config) -> color_eyre::Result<Self> {
        let mut file = std::fs::File::open(path)?;

        let mut content = String::new();

        let _ = file.read_to_string(&mut content)?;

        Ok(Self {
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
        let content = Paragraph::new(self.content.clone())
            .wrap(Wrap { trim: true })
            .style(self.styles.text_style);

        Widget::render(content, area, buf);
    }

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<ui::Message> {
        match key.code {
            crossterm::event::KeyCode::Char('Q' | 'q') => Some(ui::Message::Quit),
            crossterm::event::KeyCode::Char('F' | 'f') => Some(ui::Message::SwitchSelect),

            _ => None,
        }
    }
}
