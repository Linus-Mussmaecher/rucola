/// Copyright (C) 2024 Linus Mussmaecher <linus.mussmaecher@gmail.com>
use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::panic;

/// The actual application, combining the ui and data management.
mod app;
/// Data manipulation: Reading, parsing and manipulating note files and calculating statistics.
mod data;
/// Error enum and handling.
mod error;
/// Interaction with the file system & configuration.
mod io;
/// The ui of the app.
mod ui;
use clap::Parser;
mod config;
pub use config::Config;

/// Command line arguments for the Rucola markdown note management program.
/// Copyright (C) 2024 Linus Mussmaecher <linus.mussmaecher@gmail.com>.
/// This program comes with ABSOLUTELY NO WARRANTY.
/// This is free software, and you are welcome to redistribute it under certain conditions.
/// Type `rucola --license` for details.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    /// Target vault folder, containing the notes to index and manage. Overrides the path set in the config file.
    target_folder: Option<String>,
    /// A path to a file (relative to the config directory) containing the styles to use for the UI
    #[arg(short, long)]
    style: Option<String>,
    /// Output the license and warranty.
    #[arg(short, long)]
    license: bool,
}

/// Main function
fn main() -> error::Result<()> {
    // === Read command line arguments
    let args = Arguments::parse();

    // === Help Notices etc. ===
    if args.license {
        print_license();
        return Ok(());
    }

    // === Actual programm ===

    // Initialize hooks & terminal (ratatui boilerplate)
    init_hooks()?;
    let mut terminal = init_terminal()?;

    // draw loading screen
    draw_loading_screen(&mut terminal)?;

    // Create the app state
    let (mut app, errors) = app::App::new(args);

    // Displayed error
    let mut current_error: Option<error::RucolaError> = errors.into_iter().last();

    // Main loop
    'main: loop {
        // Draw the current screen.
        terminal.draw(|frame: &mut Frame| {
            let area = frame.size();
            let buf = frame.buffer_mut();

            // Make sure area is large enough or show error
            if area.width < 90 || area.height < 25 {
                // area too small and no error -> show area error
                current_error = Some(error::RucolaError::SmallArea);
            }

            let app_area = match &current_error {
                // If there is an error to be displayed
                Some(e) => {
                    // Separate the usual app area into a small bottom line for the area and a big area for what can be displayed of the app.
                    let areas =
                        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

                    // Render the error to the bottom.
                    Widget::render(e.to_ratatui(), areas[1], buf);

                    // Return the rest of the area for the app to render in.
                    areas[0]
                }
                // No error => App can render in the entire area.
                None => area,
            };

            // Draw the actual application
            app.draw(app_area, buf);
        })?;

        // Inform the app of events
        let maybe_keypress = if event::poll(std::time::Duration::from_millis(500))? {
            // Some event => reset current error
            current_error = None;
            // Check if the event was a keypress
            match event::read()? {
                event::Event::Key(key) if key.kind == event::KeyEventKind::Press => Some(key),
                _ => None,
            }
        } else {
            None
        };

        // update the app and deal with messages
        match app.update(maybe_keypress) {
            Ok(ui::TerminalMessage::Quit) => {
                break 'main;
            }
            Ok(ui::TerminalMessage::None) => {}
            Ok(ui::TerminalMessage::OpenExternalCommand(mut cmd)) => {
                // Restore the terminal
                restore_terminal()?;
                // Execute the given command
                cmd.status()?;
                // Re-enter the tui state
                terminal = init_terminal()?;
            }
            Err(e) => current_error = Some(e),
        }
    }

    //Restore previous terminal state
    restore_terminal()?;

    // Return the right OK
    Ok(())
}

/// Ratatui boilerplate to set up panic hooks
fn init_hooks() -> error::Result<()> {
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
fn init_terminal() -> std::io::Result<Terminal<impl ratatui::backend::Backend>> {
    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

/// Ratatui boilerplate to restore the terminal to a usable state after program exits (regularly or by panic)
fn restore_terminal() -> std::io::Result<()> {
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Prints license information to the console.
fn print_license() {
    print!("Rucola is released under the GNU General Public License v3, available at <https://www.gnu.org/licenses/gpl-3.0>.

Copyright (C) 2024 Linus Mußmächer <linus.mussmaecher@gmail.com>

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
    ");
}

/// Draws nothing but a loading screen with an indexing message.
/// Temporary screen while the programm is indexing.
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
