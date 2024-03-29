use std::io::Read;

use crate::config;
use crate::parser;
use crate::ui;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

pub struct DisplayScreen {
    styles: ui::MdStyles,
    tokens: Vec<parser::MdToken>,
}

impl DisplayScreen {
    pub fn new(path: &std::path::Path, config: &config::Config) -> color_eyre::Result<Self> {
        let mut file = std::fs::File::open(path)?;

        let mut content = String::new();

        let _ = file.read_to_string(&mut content)?;

        Ok(Self {
            styles: config.get_md_styles().clone(),
            tokens: parser::parse_note(&content),
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
            // take the markdown tokens
            .tokens
            .iter()
            // split them by linebreaks
            .group_by(|token| token.is_line_break())
            .into_iter()
            // now, iterator over all created groups
            .flat_map(|(is_line_break, group)| {
                // check if its 'just' the line break separator
                match is_line_break {
                    // yes -> return none, will be skipped by flat_map
                    true => None,
                    false =>
                    // no -> create a Line from the group
                    {
                        Some(Line::from(
                            // by iterating over the contained tokens
                            group
                                .into_iter()
                                // and converting them to a styled ratatui span with the provided method
                                .map(|token| token.to_span(&self.styles))
                                // then collect to a vec for Line to take in
                                .collect::<Vec<_>>(),
                        ))
                    }
                }
            })
            // Collect a vec of lines
            .collect::<Vec<_>>();

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
