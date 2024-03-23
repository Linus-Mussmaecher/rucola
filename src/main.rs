use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::{collections::HashMap, io::stdout, rc::Rc};

mod config;
mod data;
mod ui;

/// The main state of the application.
struct App {
    /// The currently displayed UI screen.
    screen: Box<dyn ui::Screen>,
    /// All notes managed by the application, keyed by their ID.
    index: Rc<HashMap<String, data::Note>>,
}

fn main() -> color_eyre::Result<()> {
    // Initialize terminal

    let mut terminal = init_hooks_and_terminal()?;

    // Draw 'loading' screen
    terminal.draw(|frame| {
        frame.render_widget(
            ratatui::widgets::Paragraph::new("Indexing..."),
            frame.size(),
        );
    })?;

    // Initialize state

    let config = config::Config::load()?;

    let index = Rc::new(data::create_index(std::path::Path::new(
        &config.get_vault_path(),
    ))?);

    let mut app = App {
        screen: Box::new(ui::screen::SelectScreen::new(index.clone(), &config)),
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
                                app.screen = Box::new(ui::screen::SelectScreen::new(
                                    app.index.clone(),
                                    &config,
                                ));
                            }
                            ui::Message::SwitchDisplay(path) => {
                                if let Ok(loaded_note) =
                                    ui::screen::DisplayScreen::new(&path, &config)
                                {
                                    app.screen = Box::new(loaded_note);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Save configuration
    config.store()?;

    //Restore previous terminal state (also returns Ok(()), so we can return that up if nothing fails)
    restore_terminal()
}

fn init_hooks_and_terminal() -> color_eyre::Result<Terminal<impl ratatui::backend::Backend>> {
    // Step 1: Set panic hooks (ratatui tutorial boilerplate)

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

    // Step 2: Initialize terminal

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    // Step -1: More ratatuio boilerplate that happens on a panic to make sure the terminal remains usable
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
