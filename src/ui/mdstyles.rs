use ratatui::style::{Modifier, Style};

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MdStyles {
    pub heading: Style,
    pub heading_size: u8,
}

impl Default for MdStyles {
    fn default() -> Self {
        Self {
            heading: Style::new()
                .fg(ratatui::style::Color::Blue)
                .add_modifier(Modifier::BOLD),
            heading_size: 24,
        }
    }
}
