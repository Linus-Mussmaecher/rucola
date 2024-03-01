use crossterm::event::KeyCode;

mod main_screen;
use crate::ui::input_manager;
use ratatui::{widgets::Block, Frame};

/// A trait that is implemented by different screens within the application.
pub trait Screen {
    /// Draws the screen to the frame (taking all the available space).
    fn draw(&self, frame: &mut Frame);

    /// Informs the screen of user inputs and possibly modifies the content based on this and past inputs.
    fn handle_input(&mut self, key: KeyCode) -> Option<Box<dyn Screen>>;
}

pub struct TestScreen {
    thing: String,
    manager: input_manager::SequenceManager,
}

impl TestScreen {
    pub fn new() -> Self {
        Self {
            thing: String::new(),
            manager: input_manager::SequenceManager::new(),
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

    fn handle_input(&mut self, key: KeyCode) -> Option<Box<dyn Screen>> {
        if !self.manager.register(&key) {
            if key == KeyCode::Enter {
                self.thing = format!(
                    "{}{}",
                    self.thing,
                    self.manager.get_sequence().iter().collect::<String>()
                );
                self.manager.reset_sequence();
            }
            if key == KeyCode::Backspace {
                self.thing.clear();
            }
        }
        None
    }
}
