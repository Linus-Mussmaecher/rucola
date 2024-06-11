use ratatui::{buffer, layout};

mod select_screen;
pub use select_screen::SelectScreen;
pub use select_screen::StatsShow;

mod display_screen;
pub use display_screen::DisplayScreen;

use crate::{error, ui};

/// A trait that is implemented by different screens within the application.
pub trait Screen {
    /// Draws the screen to the frame (taking all the available space).
    fn draw(&self, area: layout::Rect, buf: &mut buffer::Buffer);

    /// Informs the screen of user messages and possibly modifies the content.
    fn update(&mut self, key: crossterm::event::KeyEvent) -> error::Result<ui::Message>;
}
