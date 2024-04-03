/// Struct that describes an block of markdown (e.g. a block of text, a heading or an $$-enclosed LaTeX-Block), with its type and formatting.
pub struct MdBlock {
    /// The clean, unformated content
    content: String,
    /// The type and accompanying data.
    block_type: MdBlockType,
}

impl MdBlock {
    /// Converts a string slice into a markdown block of (formatted) text.
    /// Returns None if the buffer is empty.
    pub fn extract_text(buffer: &str) -> Option<Self> {
        if buffer.is_empty() {
            return None;
        }
        Some(Self {
            content: buffer.to_owned(),
            block_type: MdBlockType::Text,
        })
    }

    /// Converts a string slice into a markdown heading block.
    /// Returns None if the buffer empty.
    pub fn extract_heading(buffer: &str) -> Option<Self> {
        if buffer.is_empty() {
            return None;
        }
        Some(Self {
            content: buffer.trim_start_matches(['#', ' ']).to_owned(),
            block_type: MdBlockType::Heading,
        })
    }

    /// Converts a string slice into a markdown latex block, with SVG.
    /// Returns None if the buffer empty.
    pub fn extract_latex(buffer: &str) -> Option<Self> {
        if buffer.is_empty() {
            return None;
        }
        Some(Self {
            content: buffer.to_owned(),
            block_type: MdBlockType::Latex,
        })
    }
    /// Converts a string slice into a markdown latex block, mostly unformated.
    /// Returns None if the buffer empty.
    pub fn extract_code(buffer: &str) -> Option<Self> {
        if buffer.is_empty() {
            return None;
        }
        Some(Self {
            content: buffer.to_owned(),
            block_type: MdBlockType::Code,
        })
    }
}

/// Inner enum of MdBlock
enum MdBlockType {
    /// A heading.
    Heading,
    /// A paragraph of text.
    Text,
    /// A separate LaTeX block, not inline.
    Latex,
    /// A separate Code block, not inline.
    Code,
}
