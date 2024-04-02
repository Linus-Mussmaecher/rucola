pub struct MdBlock {}

impl MdBlock {
    pub fn new(mdbtype: MdBlockType, content: String) -> Self {
        Self {}
    }
}

/// Describes the type of a line or block in an MdFile.
pub enum MdBlockType {
    /// A line just containing a new line.
    Newline,
    /// A line/block containing just running text to be parsed.
    Text,
    /// A line containing a heading.
    Heading,
    /// A line containing code or the opening of a  code block.
    Code,
    /// A line containing latex code or the opening of a latex block.
    LaTeX,
}

impl MdBlockType {
    /// Takes a line and returns the type of markdown block it belongs to.
    /// Does not consider context, so a line of LaTeX between two `$$` lines will be recognized as 'Text'.
    pub fn recognize_line(line: &str) -> Self {
        // Just a new line
        if line == "\n" {
            return MdBlockType::Newline;
        }

        // Start or end of a code block
        if line.starts_with("````") {
            return MdBlockType::Code;
        }

        // Start or end of a latex block
        if line.starts_with("$$") {
            return MdBlockType::LaTeX;
        }

        // Heading line
        if regex::Regex::new(r"[\#]+\s")
            .expect("Static regex did not compile.")
            .is_match(line)
        {
            return MdBlockType::Heading;
        }

        // Everything else: Normal for now
        MdBlockType::Text
    }
}
