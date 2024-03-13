use std::io::Read;

use crate::ui::input::Message;
use ratatui::{prelude::*, widgets::*};

pub struct DisplayScreen {
    content: String,
}

impl DisplayScreen {
    pub fn new(path: &std::path::Path) -> color_eyre::Result<Self> {
        let mut file = std::fs::File::open(path)?;

        let mut content = String::new();

        let _ = file.read_to_string(&mut content)?;

        Ok(Self { content })
    }
}

impl super::Screen for DisplayScreen {
    fn draw(
        &self,
        area: ratatui::prelude::layout::Rect,
        buf: &mut ratatui::prelude::buffer::Buffer,
    ) {
        let content = Paragraph::new(self.content.clone()).wrap(Wrap { trim: true });

        Widget::render(content, area, buf);
    }

    fn update(&mut self, key: crossterm::event::KeyEvent) -> Option<Message> {
        match key.code {
            crossterm::event::KeyCode::Char('Q' | 'q') => Some(Message::Quit),
            crossterm::event::KeyCode::Char('F' | 'f') => Some(Message::SwitchSelect),

            _ => None,
        }
    }
}
