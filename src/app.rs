use super::{data, error, io, ui, ui::Screen};
use ratatui::prelude::*;

/// The main state of the application.
/// Consists of a select screen that is always existent, a stack of notes the user has navigated through and that he can navigate through by popping, reversing its navigation. Lastly, there is a display screen of the currently displayed note, which should always correspond to the top of the stack.
pub struct App {
    /// The currently displayed UI screen.
    select: ui::screen::SelectScreen,
    /// The top of the display stack, if present.
    display: Option<ui::screen::DisplayScreen>,
    /// The ids of note on the display stack
    display_stack: Vec<String>,
    /// Index note data
    index: data::NoteIndexContainer,

    // === Config ===
    /// The file manager this app's screens use to enact the user's file system requests on the file system.
    manager: io::FileManager,
    /// The HtmlBuider this app's screens use to continuously build html files.
    builder: io::HtmlBuilder,
    /// The styles used by this app's screens.
    styles: ui::UiStyles,
}

impl App {
    /// Creates a new application state. This includes
    ///  - Loading a config file
    ///  - Indexing notes from the given path
    ///  - Creating an initial select screen and empty display stack
    /// Also returns all errors that happened during creation that did not prevent the creation.
    pub fn new(args: crate::Arguments) -> (Self, Vec<error::RucolaError>) {
        // Gather errors
        let mut errors = vec![];

        let (config, vault_path) = match crate::Config::load(args) {
            Ok(config_data) => config_data,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };

        let styles = match ui::UiStyles::load(&config) {
            Ok(config) => config,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };

        let builder = io::HtmlBuilder::new(&config, vault_path.clone());

        let manager = io::FileManager::new(&config, vault_path.clone());

        let tracker = match io::FileTracker::new(&config, vault_path) {
            Ok(config) => config,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };

        // Index all files in path
        let (index, index_errors) = data::NoteIndex::new(tracker, builder.clone());
        errors.extend(index_errors);

        let index = std::rc::Rc::new(std::cell::RefCell::new(index));

        // Initialize app state
        (
            Self {
                select: ui::screen::SelectScreen::new(
                    index.clone(),
                    manager.clone(),
                    builder.clone(),
                    styles.clone(),
                    config.stats_show,
                ),
                display: None,
                display_stack: Vec::new(),
                index,
                styles,
                manager,
                builder,
            },
            errors,
        )
    }

    /// Reads the top of the display stack, creates a new display screen from it and sets that as the currently active display screen.
    /// If the display stack is empty, clears the display screen.
    fn set_display_to_top(&mut self) -> error::Result<()> {
        self.display = match self.display_stack.last() {
            Some(id) => Some(ui::screen::DisplayScreen::new(
                id,
                self.index.clone(),
                self.manager.clone(),
                self.builder.clone(),
                self.styles.clone(),
            )?),
            None => None,
        };
        Ok(())
    }

    // Updates the app with the given key.
    pub fn update(
        &mut self,
        key: Option<crossterm::event::KeyEvent>,
    ) -> error::Result<ui::TerminalMessage> {
        // Check for file changes
        let mut index = self.index.borrow_mut();
        let modifications = index.handle_file_events()?;
        drop(index);

        // if anything happened in the file system, better refresh the environment
        if modifications {
            self.select.refresh_env_stats();
            // if we are currently in a display screen, also refresh it (by creating it anew)
            if self.display.is_some() {
                self.set_display_to_top()?;
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
                self.set_display_to_top()?;
            }
            ui::Message::DisplayStackPush(new_id) => {
                // Push a new id on top of the display stack.
                self.display_stack.push(new_id.clone());
                self.set_display_to_top()?;
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
