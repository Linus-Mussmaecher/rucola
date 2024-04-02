pub struct MdBlock {}

impl MdBlock {
    pub fn new(mdbtype: MdBlockType, content: String) -> Self {
        Self {}
    }
}

pub enum MdBlockType {
    Newline,
    Text,
    Heading,
    Code,
    LaTeX,
}

impl MdBlockType {
    /// Takes a line and returns the type of markdown block it belongs to.
    /// Does not consider context, so a line of LaTeX between two `$$` lines will be recognized as 'Text'.
    pub fn recognize_line(line: &str) -> Self {
        if line == "\n" {
            return MdBlockType::Newline;
        }

        if line.starts_with("````") {
            return MdBlockType::Code;
        }

        if line.starts_with("$$") {
            return MdBlockType::LaTeX;
        }

        if regex::Regex::new(r"[\#]+\s")
            .expect("Static regex did not compile.")
            .is_match(line)
        {
            return MdBlockType::Heading;
        }

        MdBlockType::Text
    }
}
