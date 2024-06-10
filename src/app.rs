use super::{config, data, error, ui, ui::Screen};
use notify::{self, Watcher};
use ratatui::prelude::*;
use std::{cell::RefCell, rc::Rc, sync::mpsc};

/// The main state of the application.
/// Consists of a select screen that is always existent, a stack of notes the user has navigated through and that he can navigate through by popping, reversing its navigation. Lastly, there is a display screen of the currently displayed note, which should always correspond to the top of the stack.
pub struct App {
    /// The currently displayed UI screen.
    select: ui::screen::SelectScreen,
    /// The top of the display stack, if present.
    display: Option<ui::screen::DisplayScreen>,
    /// The ids of note on the display stack
    display_stack: Vec<String>,
    /// Stored config data
    config: config::Config,
    /// Index note data
    index: data::NoteIndexContainer,
    /// Watcher that checks for file changes in the vault directory and needs to be kept alive with this index.
    /// Can be unused because it is just here for RAII.
    #[allow(unused)]
    watcher: notify::INotifyWatcher,
    /// Channel from which file change events in the vault directory are deposited by the watcher and can be requested.
    file_change_channel: mpsc::Receiver<Result<notify::Event, notify::Error>>,
}

impl App {
    /// Creates a new application state. This includes
    ///  - Loading a config file
    ///  - Indexing notes from the given path
    ///  - Creating an initial select screen and empty display stack
    pub fn new(config: config::Config) -> Self {
        // Index all files in path
        let index = Rc::new(RefCell::new(data::NoteIndex::new(&config)));

        // Create asynchronous channel for file events.
        let (sender, receiver) = mpsc::channel();

        // Create watcher so we can store it in the file, delaying its drop (which stops its function) until the end of the lifetime of this index.
        let mut watcher = notify::recommended_watcher(move |res| {
            sender.send(res).unwrap();
        })
        .unwrap();

        // Start watching the vault.
        watcher
            .watch(
                &config.create_vault_path(),
                notify::RecursiveMode::Recursive,
            )
            .expect("Fixed config does not fail.");

        // Initialize app state
        Self {
            select: ui::screen::SelectScreen::new(index.clone(), &config),
            display: None,
            display_stack: Vec::new(),
            index,
            watcher,
            file_change_channel: receiver,
            config,
        }
    }

    // Updates the app with the given key.
    pub fn update(
        &mut self,
        key: Option<crossterm::event::KeyEvent>,
    ) -> Result<ui::TerminalMessage, error::RucolaError> {
        // Check for file changes
        let mut modifications = false;
        let mut index = self.index.borrow_mut();
        for event in self.file_change_channel.try_iter().flatten() {
            modifications |= index.handle_file_event(event, &self.config)?;
        }
        drop(index);

        // if anything happened in the file system, better refresh the environment
        if modifications {
            self.select.refresh_env_stats();
            // if we are currently in a display screen, also refresh it (by creating it anew)
            if self.display.is_some() {
                self.display = match self.display_stack.last() {
                    Some(id) => Some(ui::screen::DisplayScreen::new(
                        id,
                        self.index.clone(),
                        &self.config,
                    )?),
                    None => None,
                };
            }
        }

        if key.is_none() {
            return Ok(ui::TerminalMessage::None);
        }
        let key = key.expect("This not to be none, because we checked it 3 lines earlier.");

        // Update appropriate screen
        let msg = if let Some(display) = &mut self.display {
            display.update(key)
        } else {
            self.select.update(key)
        };

        let msg = msg?;

        // Act on the potentially returned message.
        match &msg {
            // Message that do not modify the app trigger no immediate effect and are later passed up.
            ui::Message::None | ui::Message::Quit | ui::Message::OpenExternalCommand(_) => {}
            ui::Message::DisplayStackClear => {
                // Clear the display stack and remove the current display screen, if there is one.
                self.display_stack.clear();
                self.display = None;
            }
            ui::Message::DisplayStackPop => {
                // Pop the top of the stack - which should correspond to the currently displayed note.
                self.display_stack.pop();
                // Attempt to read the top of the stack again.
                // Replace the display screen with the one created from this result - either a valid display screen that will be displayed or none, causing the select screen to show.
                self.display = match self.display_stack.last() {
                    Some(id) => Some(ui::screen::DisplayScreen::new(
                        id,
                        self.index.clone(),
                        &self.config,
                    )?),
                    None => None,
                };
            }
            ui::Message::DisplayStackPush(new_id) => {
                // Push a new id on top of the display stack.
                self.display_stack.push(new_id.clone());

                // Attempt to read the top of the stack again.
                // Replace the display screen with the one created from this result, which should always be a valid display screen created from the id we just pushed.
                self.display = match self.display_stack.last() {
                    Some(id) => Some(ui::screen::DisplayScreen::new(
                        id,
                        self.index.clone(),
                        &self.config,
                    )?),
                    None => None,
                };
            }
        }

        Ok(msg.into())
    }

    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        if let Some(display) = &self.display {
            display.draw(area, buf);
        } else {
            self.select.draw(area, buf);
        }
    }
}
