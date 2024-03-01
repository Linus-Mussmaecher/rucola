use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{stdout, Result};

mod data;
mod ui;

/// The main state of the application.
struct App {
    screen: Box<dyn ui::Screen>,
}

fn main() -> Result<()> {
    // All the Ratatui boilerplate.
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Initialize state
    let mut app = App {
        screen: Box::new(ui::screen::TestScreen::new()),
    };

    loop {
        // Draw the current screen.
        terminal.draw(|frame| {
            app.screen.draw(frame);
        })?;

        // Inform the current screen of events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                // Check for key preses
                if key.kind == KeyEventKind::Press {
                    // Inform the current screen.
                    app.screen.handle_input(key.code);
                    // Then quit the application.
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
