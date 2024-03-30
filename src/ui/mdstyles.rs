use ratatui::style::{Color, Modifier, Style};

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MdStyles {
    pub text: Style,
    pub heading: Style,
    pub heading_size: u8,
    pub tag: Style,
    pub star: Style,
    pub underscore: Style,
    pub doublestar: Style,
}

impl Default for MdStyles {
    fn default() -> Self {
        Self {
            text: Style::new(),
            heading: Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
            heading_size: 24,
            tag: Style::new()
                .bg(Color::LightBlue)
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
            star: Style::new().add_modifier(Modifier::ITALIC),
            underscore: Style::new().add_modifier(Modifier::ITALIC),
            doublestar: Style::new().add_modifier(Modifier::BOLD),
        }
    }
}
