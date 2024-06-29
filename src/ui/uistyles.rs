use ratatui::style::*;

use crate::error;

/// A struct that holds a collection of styles for a consistent looking UI.
/// This is a pure data struct, having no methods and only public attributes.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UiStyles {
    /// For titles of boxes.
    pub title_style: Style,
    /// For table headers etc.
    pub subtitle_style: Style,
    /// For letters that indicate a hotkey within a title.
    pub hotkey_style: Style,
    /// For normal text.
    pub text_style: Style,
    /// For selected list/table rows or other text.
    pub selected_style: Style,
    /// For text in an input area.
    pub input_style: Style,
}

impl Default for UiStyles {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .fg(ratatui::style::Color::LightBlue)
                .add_modifier(Modifier::BOLD),
            subtitle_style: Style::new()
                .fg(ratatui::style::Color::LightBlue)
                .add_modifier(Modifier::ITALIC),
            hotkey_style: Style::new()
                .fg(ratatui::style::Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            text_style: Style::new().fg(Color::White),
            selected_style: Style::new()
                .bg(ratatui::style::Color::Blue)
                .add_modifier(Modifier::BOLD),
            input_style: Style::new().add_modifier(Modifier::ITALIC),
        }
    }
}

impl UiStyles {
    /// Loads the style file defined in the given config file
    pub fn load(config: &crate::Config) -> error::Result<Self> {
        let uistyles: Self = confy::load("rucola", config.theme.as_str())?;
        Ok(uistyles)
    }
}
