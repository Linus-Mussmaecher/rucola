use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::{cell::RefCell, io::stdout, rc::Rc};

mod config;
mod data;
mod ui;
use ui::{screen, Screen};
/// The main state of the application.
struct App {
    /// The currently displayed UI screen.
    select: screen::SelectScreen,
    /// The top of the display stack, if present.
    display: Option<screen::DisplayScreen>,
    /// The ids of note on the display stack
    display_stack: Vec<String>,
}

/// Main function
fn main() -> color_eyre::Result<()> {
    // Initialize hooks & terminal
    init_hooks()?;
    let mut terminal = init_terminal()?;

    // draw loading screen
    draw_loading_screen(&mut terminal)?;

    // Read config file. Loading includes listening to command line.
    let config = config::Config::load().unwrap_or_default();

    // Index all files in path
    let index = Rc::new(RefCell::new(data::NoteIndex::new(
        &std::path::Path::new(&config.get_vault_path()),
        &config,
    )));

    // Initialize app state
    let mut app = App {
        select: screen::SelectScreen::new(index.clone(), &config),
        display: None,
        display_stack: Vec::new(),
    };

    // Main loop

    'main: loop {
        // Draw the current screen.
        terminal.draw(|frame: &mut Frame| {
            let area = frame.size();
            let buf = frame.buffer_mut();
            if let Some(display) = &app.display {
                display.draw(area, buf);
            } else {
                app.select.draw(area, buf);
            }
        })?;

        // Inform the current screen of events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                // Check for key preses
                if key.kind == KeyEventKind::Press {
                    // Update appropriate screen
                    let msg = if let Some(display) = &mut app.display {
                        display.update(key)
                    } else {
                        app.select.update(key)
                    };
                    // Act on the potentially returned message.
                    if let Some(msg) = msg {
                        match msg {
                            ui::Message::Quit => break 'main,
                            ui::Message::OpenExternalCommand(mut command) => {
                                // Restore the terminal
                                restore_terminal()?;
                                // Execute the given command
                                command.status()?;
                                // Re-enter the application
                                terminal = init_terminal()?;
                            }
                            ui::Message::DisplayStackClear => {
                                app.display_stack.clear();
                                app.display = None;
                            }
                            ui::Message::DisplayStackPop => {
                                app.display_stack.pop();
                                app.display = app.display_stack.last().and_then(|id| {
                                    screen::DisplayScreen::new(id, index.clone(), &config)
                                })
                            }
                            ui::Message::DisplayStackPush(new_id) => {
                                app.display_stack.push(new_id);

                                app.display = app.display_stack.last().and_then(|id| {
                                    screen::DisplayScreen::new(id, index.clone(), &config)
                                })
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

fn draw_loading_screen(
    terminal: &mut Terminal<impl ratatui::backend::Backend>,
) -> Result<CompletedFrame, std::io::Error> {
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
