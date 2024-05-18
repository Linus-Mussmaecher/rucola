use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io;
use std::panic;

mod app;
mod config;
mod data;
mod error;
mod ui;

/// Main function
fn main() -> Result<(), error::RucolaError> {
    // Initialize hooks & terminal
    init_hooks()?;
    let mut terminal = init_terminal()?;

    // draw loading screen
    draw_loading_screen(&mut terminal)?;

    // Displayed error
    let mut current_error: Option<error::RucolaError> = None;

    // Read config file. Loading includes listening to command line.
    let config = match config::Config::load() {
        Ok(conf) => conf,
        Err(e) => {
            current_error = Some(e);
            config::Config::default()
        }
    };

    // Create the app state
    let mut app = app::App::new(config);

    // Main loop
    'main: loop {
        // Draw the current screen.
        terminal.draw(|frame: &mut Frame| {
            let area = frame.size();
            let buf = frame.buffer_mut();

            // Make sure area is large enough or show error
            if (area.width < 90 || area.height < 25) && current_error.is_none() {
                current_error = Some(error::RucolaError::SmallArea)
            }

            let app_area = match &current_error {
                // If there is an error to be displayed, reduce the size for the app and display it at the bottom.
                Some(e) => {
                    let areas =
                        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

                    Widget::render(e.to_ratatui(), areas[1], buf);

                    areas[0]
                }
                None => area,
            };

            // Draw the actual application
            app.draw(app_area, buf);
        })?;

        // Inform the current screen of events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                // when a key event, first reset the current error
                current_error = None;
                // Then update the app.
                match app.update(key) {
                    Ok(ui::TerminalMessage::Quit) => {
                        break 'main;
                    }
                    Ok(ui::TerminalMessage::None) => {}
                    Ok(ui::TerminalMessage::OpenExternalCommand(mut cmd)) => {
                        // Restore the terminal
                        restore_terminal()?;
                        // Execute the given command
                        cmd.status()?;
                        // Re-enter the selflication
                        terminal = init_terminal()?;
                    }
                    Err(e) => current_error = Some(e),
                }
            }
        }
    }

    //Restore previous terminal state (also returns Ok(()), so we can return that up if nothing fails)
    restore_terminal()?;

    Ok(())
}

/// Draws nothing but a loading screen with an indexing message.
/// Temporary screen while the programm is indexing.
fn draw_loading_screen(
    terminal: &mut Terminal<impl ratatui::backend::Backend>,
) -> Result<CompletedFrame, io::Error> {
    // Draw 'loading' screen
    terminal.draw(|frame| {
        frame.render_widget(
            ratatui::widgets::Paragraph::new("Indexing...").alignment(Alignment::Center),
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .split(frame.size())[1],
        );
    })
}

/// Ratatui boilerplate to set up panic hooks
fn init_hooks() -> Result<(), error::RucolaError> {
    // Get a default panic hook
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        // Just restore the terminal.
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
    Ok(())
}

/// Ratatui boilerplate to put the terminal into a TUI state
fn init_terminal() -> io::Result<Terminal<impl ratatui::backend::Backend>> {
    io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

/// Ratatui boilerplate to restore the terminal to a usable state after program exits (regularly or by panic)
fn restore_terminal() -> io::Result<()> {
    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
