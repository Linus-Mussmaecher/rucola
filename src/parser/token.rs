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
    /// Text surrouded by stars
    Stars,
    /// Text surrounded by underscores
    Underscores,
    /// Text surrounded by double stars
    DoubleStars,
}

impl MdTokenType {
    /// Returns the Preference of this token type to be grouped with others on a single line.
    pub fn to_line_preference(&self) -> MdTokenTypeLinePreference {
        match self {
            MdTokenType::LineBreak | MdTokenType::Heading(_) => MdTokenTypeLinePreference::Alone,
            MdTokenType::DoubleStars
            | MdTokenType::Underscores
            | MdTokenType::Stars
            | MdTokenType::Text
            | MdTokenType::Tag => MdTokenTypeLinePreference::Text,
        }
    }
}

/// Describes wether a token wants to be on its own line or grouped with others.
#[derive(Clone, Copy, Debug, Eq)]
pub enum MdTokenTypeLinePreference {
    /// Always on its own line.
    Alone,
    /// Can be on the same line as others of this type.
    Text,
}

impl PartialEq for MdTokenTypeLinePreference {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MdTokenTypeLinePreference::Alone, MdTokenTypeLinePreference::Alone) => false,
            (MdTokenTypeLinePreference::Alone, MdTokenTypeLinePreference::Text) => false,
            (MdTokenTypeLinePreference::Text, MdTokenTypeLinePreference::Alone) => false,
            (MdTokenTypeLinePreference::Text, MdTokenTypeLinePreference::Text) => true,
        }
    }
}

/// Represents a single markdown token, e.g. a tag, heading, line, etc.
#[derive(Clone, Debug)]
pub struct MdToken {
    token_type: MdTokenType,
    content: String,
}

impl MdToken {
    /// Clones the provided str to create a token with the provided type
    pub fn new(token_type: MdTokenType, content: impl Into<Option<String>>) -> Self {
        Self {
            token_type,
            content: content.into().unwrap_or_default(),
        }
    }

    /// Returns the Preference of this token to be grouped with others on a single line.
    pub fn to_line_preference(&self) -> MdTokenTypeLinePreference {
        self.token_type.to_line_preference()
    }

    pub fn is_line_break(&self) -> bool {
        self.token_type == MdTokenType::LineBreak
    }

    pub fn to_span<'a>(&'a self, styles: &ui::MdStyles) -> Span<'a> {
        match self.token_type {
            // this specifically should never be called.
            MdTokenType::LineBreak => Span::raw(""),
            MdTokenType::Text => Span::styled(&self.content, styles.text),
            MdTokenType::Heading(_layer) => Span::styled(&self.content, styles.heading),
            MdTokenType::Tag => Span::styled(&self.content, styles.tag),
            MdTokenType::Stars => Span::styled(&self.content, styles.star),
            MdTokenType::Underscores => Span::styled(&self.content, styles.underscore),
            MdTokenType::DoubleStars => Span::styled(&self.content, styles.doublestar),
        }
    }
}
