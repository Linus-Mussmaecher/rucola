use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::{io::stdout, rc::Rc};

mod config;
mod data;
mod ui;
use ui::screen;
/// The main state of the application.
struct App {
    /// The currently displayed UI screen.
    screen: Box<dyn ui::Screen>,
    /// All notes managed by the application, keyed by their ID.
    index: Rc<data::NoteIndex>,
}

/// Main function
fn main() -> color_eyre::Result<()> {
    // Initialize hooks & terminal
    init_hooks()?;
    let mut terminal = init_terminal()?;

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
    })?;

    // Read config file. Loading includes listening to command line.
    let config = config::Config::load().unwrap_or_default();

    // Index all files in path
    let index = Rc::new(data::NoteIndex::new(&std::path::Path::new(
        &config.get_vault_path(),
    )));

    // Initialize app state
    let mut app = App {
        screen: Box::new(screen::SelectScreen::new(index.clone(), &config)),
        index,
    };

    // Main loop

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
                    if let Some(msg) = app.screen.update(key) {
                        match msg {
                            ui::Message::Quit => break 'main,
                            ui::Message::SwitchSelect => {
                                // Replace the screen with the basic selector
                                app.screen =
                                    Box::new(screen::SelectScreen::new(app.index.clone(), &config));
                            }
                            ui::Message::SwitchDisplay(id) => {
                                // check if the note actually can be found
                                if let Some(loaded_note) =
                                    screen::DisplayScreen::new(&id, app.index.clone(), &config)
                                {
                                    // if yes, replace the current screen
                                    app.screen = Box::new(loaded_note);
                                }
                            }
                            ui::Message::OpenExternalCommand(mut command) => {
                                // Restore the terminal
                                restore_terminal()?;
                                // Execute the given command
                                command.status()?;
                                // Re-enter the application
                                terminal = init_terminal()?;
                            }
                        }
                    }
                }
            }
        }
    }

    //Restore previous terminal state (also returns Ok(()), so we can return that up if nothing fails)
    restore_terminal()
}

/// Ratatui boilerplate to set up panic hooks
fn init_hooks() -> color_eyre::Result<()> {
    let (panic, error) = color_eyre::config::HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

/// Ratatui boilerplate to put the terminal into a TUI state
fn init_terminal() -> color_eyre::Result<Terminal<impl ratatui::backend::Backend>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

/// Ratatui boilerplate to restore the terminal to a usable state after program exits (regularly or by panic)
fn restore_terminal() -> color_eyre::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
