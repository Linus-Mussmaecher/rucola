use ratatui::{text::Line, widgets::Wrap};

type RParagraph<'a> = ratatui::widgets::Paragraph<'a>;

pub struct Paragraph {
    content: String,
}

impl Paragraph {
    pub fn parse_paragraph(paragraph: &str) -> impl std::iter::Iterator<Item = Self> {
        std::iter::once(Self {
            content: paragraph.to_string(),
        })
    }

    pub fn to_widget<'a>(&'a self) -> Line<'a> {
        Line::from(self.content.as_str())
    }
}
