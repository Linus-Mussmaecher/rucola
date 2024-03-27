use ratatui::style::{Modifier, Style};

/// A struct that holds a collection of styles for a consistent looking UI.
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
                .fg(ratatui::style::Color::Cyan)
                .add_modifier(Modifier::BOLD),
            subtitle_style: Style::new().fg(ratatui::style::Color::LightCyan),
            hotkey_style: Style::new()
                .fg(ratatui::style::Color::Blue)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            text_style: Style::new(),
            selected_style: Style::new()
                .bg(ratatui::style::Color::LightCyan)
                .add_modifier(Modifier::BOLD),
            input_style: Style::new().add_modifier(Modifier::ITALIC),
        }
    }
}
