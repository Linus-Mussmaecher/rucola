use super::token;
/// Parser for a single markdown token (tag, heading, italic, etc.).
/// Can recognize the pattern with regex and convert a string to an MdToken.
#[derive()]
pub struct TokenParser {
    regex: regex::Regex,
    converter: Box<dyn Fn(&str) -> token::MdToken>,
}

impl TokenParser {
    /// Creates a new token parser with the specified regex and converter.
    fn new<F>(regex: regex::Regex, converter: F) -> Self
    where
        F: 'static + Fn(&str) -> token::MdToken,
    {
        Self {
            regex,
            converter: Box::new(converter),
        }
    }

    /// Returns the regex in this parser for matching.
    pub fn get_regex(&self) -> &regex::Regex {
        &self.regex
    }

    /// Converts the specified string into the token it requires, if it matches the regex.
    /// When writing the converter, it can therefore always be assumed that all input given to it matches the regex.
    pub fn convert(&self, content: &str) -> Option<token::MdToken> {
        if self.regex.is_match(content) {
            Some((self.converter)(content))
        } else {
            None
        }
    }

    /// Creates a token parser recognizing line breaks.
    pub fn create_line_break_parser() -> Self {
        Self::new(
            regex::Regex::new(r"\n[\n]+").expect("Static regex ill-formed."),
            |_substr| token::MdToken::new(token::MdTokenType::LineBreak, None),
        )
    }
    /// Creates a token parser recognizing headings.
    pub fn create_headings_parser() -> Self {
        Self::new(
            regex::Regex::new(r"[\#]+\s[^\n\#]*\n?").expect("Static regex ill-formed."),
            |substr| {
                token::MdToken::new(
                    token::MdTokenType::Heading(substr.chars().filter(|c| *c == '#').count() as u8),
                    substr.trim_start_matches(['#', ' ']).to_string(),
                )
            },
        )
    }
    /// Creates a token parser recognizing tags.
    pub fn create_tag_parser() -> Self {
        Self::new(
            regex::Regex::new(r"\#[^\#\s]*").expect("Static regex ill-formed."),
            |substr| token::MdToken::new(token::MdTokenType::Tag, substr.trim_end().to_string()),
        )
    }
    /// Creates a token parser recognizing any text.
    pub fn create_text_parser() -> Self {
        Self::new(
            regex::Regex::new(r".*").expect("Static regex ill-formed."),
            |substr| token::MdToken::new(token::MdTokenType::Text, substr.to_string()),
        )
    }
    /// Creates a token parser recognizing text in stars.
    pub fn create_star_parser() -> Self {
        Self::new(
            regex::Regex::new(r"\*[^\*\n]*\*").expect("Static regex ill-formed."),
            |substr| {
                token::MdToken::new(
                    token::MdTokenType::Stars,
                    substr
                        .trim_start_matches('*')
                        .trim_end_matches('*')
                        .to_string(),
                )
            },
        )
    }
    /// Creates a token parser recognizing text in underscores.
    pub fn create_underscore_parser() -> Self {
        Self::new(
            regex::Regex::new(r"_[^_\n]*_").expect("Static regex ill-formed."),
            |substr| {
                token::MdToken::new(
                    token::MdTokenType::Underscores,
                    substr
                        .trim_start_matches('_')
                        .trim_end_matches('_')
                        .to_string(),
                )
            },
        )
    }
    /// Creates a token parser recognizing text in double stars.
    pub fn create_double_star_parser() -> Self {
        Self::new(
            regex::Regex::new(r"\*\*[^\*\n]*\*\*").expect("Static regex ill-formed."),
            |substr| {
                token::MdToken::new(
                    token::MdTokenType::DoubleStars,
                    substr
                        .trim_start_matches('*')
                        .trim_end_matches('*')
                        .to_string(),
                )
            },
        )
    }
}
