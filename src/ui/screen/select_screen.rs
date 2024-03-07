use std::collections::HashMap;
use std::rc::Rc;

use crate::data::Note;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

pub struct SelectScreen {
    index: Rc<HashMap<String, Note>>,
    names: Vec<String>,
}

impl SelectScreen {
    pub fn new(index: Rc<HashMap<String, Note>>) -> Self {
        Self {
            names: index
                .values()
                .map(|v| v.name.clone())
                .collect::<Vec<String>>(),
            index: index,
        }
    }
}

impl super::Screen for SelectScreen {
    fn update(&mut self, key: KeyEvent) -> Option<crate::ui::input::Message> {
        match key.code {
            KeyCode::Char('s') => return Some(crate::ui::input::Message::SwitchStats),
            KeyCode::Char('q') => return Some(crate::ui::input::Message::Quit),
            _ => {}
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

        let mut state = ListState::default();

        let list = List::new(self.names.clone())
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
