use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::{
    collections::HashMap,
    fmt::Pointer,
    io::{stdout, Result},
};

mod data;
mod ui;

/// The main state of the application.
struct App {
    screen: Box<dyn ui::Screen>,
    index: HashMap<String, data::Note>,
}

fn main() -> Result<()> {
    // TODO: Panic hooks
    // TODO: Error handling
    // All the Ratatui boilerplate.
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Initialize state

    let index = data::create_index(std::path::Path::new("/home/linus/Coppermind/"));

    let mut app = App {
        screen: Box::new(ui::screen::SelectScreen::new(index.clone())),
        index,
    };

    // Initialize input handler
    let mut handler = ui::InputManager::default();

    'main: loop {
        // Draw the current screen.
        terminal.draw(|frame| {
            let area = frame.size();
            let buf = frame.buffer_mut();

            app.screen.draw(area, buf);
        })?;

        // Inform the current screen of events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                // Check for key preses
                if key.kind == KeyEventKind::Press {
                    // Register input and get message.
                    let mut maybe_message = handler.register(key.code);
                    while let Some(msg) = maybe_message {
                        // Inform the current screen.
                        maybe_message = app.screen.update(msg);
                        // Check for quits and screen changes
                        if let Some(ui::input::Message::Quit) = maybe_message {
                            break 'main;
                        }
                    }
                }
            }
        }
    }

    //Ratatui boilerplate
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
