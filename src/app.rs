use super::{data, error, io, ui, ui::Screen};
use ratatui::prelude::*;

/// The main state of the application.
/// Consists of a select screen that is always existent, a stack of notes the user has navigated through and that he can navigate through by popping, reversing its navigation. Lastly, there is a display screen of the currently displayed note, which should always correspond to the top of the stack.
pub struct App {
    // === UI ===
    /// The current select screen (might be overlayed by a display screen and thus not rendered).
    select: ui::screen::SelectScreen,
    /// The top of the display stack, if present.
    display: Option<ui::screen::DisplayScreen>,
    /// The ids of note on the display stack
    display_stack: Vec<String>,

    // === DATA ===
    /// Index note data
    index: data::NoteIndexContainer,

    // === CONFIG ===
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
    ///
    /// Also returns all errors that happened during creation that did not prevent the creation.
    pub fn new<F: FnMut(&str) -> error::Result<()>>(
        args: crate::Arguments,
        mut loading_screen_callback: F,
    ) -> (Self, Vec<error::RucolaError>) {
        // Gather errors
        let mut errors = Vec::new();

        // Load configuration
        errors.extend(loading_screen_callback("Loading configuration...").err());

        let (config, vault_path) = match crate::Config::load(args) {
            Ok(config_data) => config_data,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };

        // Load the style file specified in the configuration
        errors.extend(loading_screen_callback("Loading styles...").err());

        let styles = match ui::UiStyles::load(&config) {
            Ok(config) => config,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };

        // Use the config file to create managers & trackers
        errors.extend(loading_screen_callback("Creating managers & trackers...").err());

        let builder = io::HtmlBuilder::new(&config, vault_path.clone());

        let manager = io::FileManager::new(&config, vault_path.clone());

        let git_manager = io::GitManager::new(vault_path.clone());

        let tracker = match io::FileTracker::new(&config, vault_path.clone()) {
            Ok(tracker) => tracker,
            Err(e) => {
                errors.push(e);
                Default::default()
            }
        };

        // Print error message based on current directory
        let mut msg = "Indexing...";
        if let Some(user_dirs) = directories::UserDirs::new() {
            if vault_path == directories::UserDirs::home_dir(&user_dirs) {
                msg = "Indexing...\n\nYou are running rucola in your home directory. This might take a while.\nConsider running in your notes directory instead.";
            }
        }

        errors.extend(loading_screen_callback(msg).err());

        // Index all files in path
        let (index, index_errors) = data::NoteIndex::new(tracker, builder.clone());
        errors.extend(index_errors);

        let index = std::rc::Rc::new(std::cell::RefCell::new(index));

        // Use the config file to create managers & trackers
        errors.extend(loading_screen_callback("Initiliazing app state...").err());

        // Initialize app state
        (
            Self {
                select: ui::screen::SelectScreen::new(
                    index.clone(),
                    manager.clone(),
                    git_manager,
                    builder.clone(),
                    styles,
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
                self.styles,
            )?),
            None => None,
        };
        Ok(())
    }

    // Updates the app with the given key.
    pub fn update(
        &mut self,
        key: Option<ratatui::crossterm::event::KeyEvent>,
    ) -> error::Result<ui::TerminalMessage> {
        // Check for file changes
        let mut index = self.index.borrow_mut();
        let (modifications, id_changes) = index.handle_file_events()?;
        drop(index);

        // synchronize display stack with id changes from file events
        for changed_id in id_changes {
            // if an id was deleted or modified, remove all such displays from the stack
            self.display_stack
                .retain(|display_id| *display_id != changed_id);
        }

        // remove 'empty' ids, indicating that
        self.display_stack
            .retain(|display_id| !display_id.is_empty());

        if modifications {
            // if anything happened in the file system, better refresh the filters
            self.select.refresh_env_stats();
            // also refresh the display by setting it to none
            self.set_display_to_top()?;
        }

        let key = if let Some(key) = key {
            key
        } else {
            return Ok(ui::TerminalMessage::None);
        };

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
