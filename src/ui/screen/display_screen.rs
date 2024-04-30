use crate::{config, data, ui};

use ratatui::{prelude::*, widgets::*};

/// The display screen displays a single note to the user.
pub struct DisplayScreen {
    /// The title of the displayed note.
    title: String,
    /// The used styling theme
    style: ui::UiStyles,
}

impl DisplayScreen {
    pub fn new(note: &data::Note, config: &config::Config) -> color_eyre::Result<Self> {
        Ok(Self {
            title: note.name.clone(),
            style: config.get_ui_styles().clone(),
        })
    }
}

impl super::Screen for DisplayScreen {
    fn draw(
        &self,
        area: ratatui::prelude::layout::Rect,
        buf: &mut ratatui::prelude::buffer::Buffer,
    ) {
        // Create a paragraph from this vec and display it.
        Widget::render(
            Paragraph::new(self.title.as_str())
                .wrap(Wrap { trim: true })
                .style(self.style.title_style),
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
