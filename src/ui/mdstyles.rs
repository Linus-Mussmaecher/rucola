use ratatui::style::{Color, Modifier, Style};

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MdStyles {
    pub text: Style,
    pub heading: Style,
    pub heading_size: u8,
    pub tag: Style,
}

impl Default for MdStyles {
    fn default() -> Self {
        Self {
            text: Style::new(),
            heading: Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
            heading_size: 24,
            tag: Style::new()
                .bg(Color::LightBlue)
                .add_modifier(Modifier::ITALIC),
        }
    }
}
