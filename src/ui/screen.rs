use crossterm::event::KeyCode;

mod main_screen;
use ratatui::{style::Stylize as _, Frame};

pub trait Screen {
    fn draw(&self, frame: &mut Frame);

    fn handle_input(&mut self, keys: &[KeyCode]) -> (bool, Option<Box<dyn Screen>>);
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
        frame.render_widget(
            ratatui::widgets::Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                .white()
                .on_blue(),
            area,
        );
    }

    fn handle_input(&mut self, keys: &[KeyCode]) -> (bool, Option<Box<dyn Screen>>) {
        (false, None)
    }
}
