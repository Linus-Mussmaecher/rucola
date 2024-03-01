mod main_screen;
pub use main_screen::MainScreen;

use crate::ui::input;

/// A trait that is implemented by different screens within the application.
pub trait Screen {
    /// Draws the screen to the frame (taking all the available space).
    fn draw(&self, frame: &mut ratatui::Frame);

    /// Informs the screen of user messages and possibly modifies the content.
    fn update(&mut self, msg: input::Message) -> Option<input::Message>;
}
