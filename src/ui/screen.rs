mod main_screen;
use crate::ui::input;
use ratatui::{widgets::Block, Frame};

/// A trait that is implemented by different screens within the application.
pub trait Screen {
    /// Draws the screen to the frame (taking all the available space).
    fn draw(&self, frame: &mut Frame);

    /// Informs the screen of user messages and possibly modifies the content.
    fn update(&mut self, msg: input::Message) -> Option<input::Message>;
}

pub struct TestScreen {
    thing: String,
}

impl TestScreen {
    pub fn new() -> Self {
        Self {
            thing: String::new(),
        }
    }
}

impl Screen for TestScreen {
    fn draw(&self, frame: &mut Frame) {
        let area = frame.size();
        let ar1 = ratatui::layout::Rect {
            height: area.height / 2,
            ..area
        };
        let ar2 = ratatui::layout::Rect {
            height: area.height / 2,
            y: area.height / 2,
            ..area
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                .block(Block::bordered()),
            ar1,
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(format!("String: {}", self.thing))
                .block(Block::bordered()),
            ar2,
        );
    }

    fn update(&mut self, msg: input::Message) -> Option<input::Message> {
        match msg {
            input::Message::GotoEnd => self.thing.push('1'),
            input::Message::Clear => self.thing.clear(),
            m => return Some(m),
        }
        None
    }
}
