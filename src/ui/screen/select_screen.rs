use std::collections::HashMap;

use crate::data::Note;
use ratatui::{prelude::*, widgets::*};

pub struct SelectScreen {
    index: HashMap<String, Note>,
}

impl SelectScreen {
    pub fn new(index: HashMap<String, Note>) -> Self {
        Self { index }
    }
}

impl super::Screen for SelectScreen {
    fn update(&mut self, msg: crate::ui::input::Message) -> Option<crate::ui::input::Message> {
        match msg {
            crate::ui::input::Message::Clear => self.index.clear(),
            m => return Some(m),
        }
        None
    }

    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer) {
        let vertical = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Min(3),
            Constraint::Percentage(70),
        ]);
        let [info_area, _search_area, list_area] = vertical.areas(area);

        // TODO: Stateful list

        let vec = self
            .index
            .clone()
            .into_values()
            .map(|v| v.name)
            .collect::<Vec<String>>();

        let mut state = ListState::default();

        let list = List::new(vec)
            .block(Block::bordered().title("Notes"))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);

        StatefulWidget::render(list, list_area, buf, &mut state);

        Widget::render(
            ratatui::widgets::Paragraph::new(format!("Lots of info"))
                .block(Block::bordered().title("Statistics")),
            info_area,
            buf,
        );
    }
}
