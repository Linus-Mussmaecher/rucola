use crate::ui;
use ratatui::text::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdTokenType {
    /// A line break in the display (not in the source!)
    LineBreak,
    /// A text
    Text,
    /// A heading (of with the layer given as the attribute)
    Heading(u8),
    /// A tag
    Tag,
}

type MTT = MdTokenType;

/// Represents a single markdown token, e.g. a tag, heading, line, etc.
#[derive(Clone, Debug)]
pub struct MdToken {
    token_type: MdTokenType,
    content: String,
}

impl MdToken {
    /// Clones the provided str to create a token with the provided type
    pub fn new(token_type: MdTokenType, content: &str) -> Self {
        Self {
            token_type,
            content: content.to_string(),
        }
    }

    pub fn is_line_break(&self) -> bool {
        self.token_type == MTT::LineBreak
    }

    pub fn to_span<'a>(&'a self, styles: &ui::MdStyles) -> Span<'a> {
        match self.token_type {
            MdTokenType::LineBreak => Span::raw(""),
            MdTokenType::Text => Span::styled(&self.content, styles.text),
            MdTokenType::Heading(_layer) => Span::styled(&self.content, styles.heading),
            MdTokenType::Tag => Span::styled(&self.content, styles.tag),
        }
    }
}
