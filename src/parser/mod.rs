mod md_block;
pub use md_block::MdBlock;

mod md_line_type;
pub use md_line_type::MdLineType;

pub fn parse_note(note: &str) -> Vec<MdBlock> {
    // Prepare buffer to collect multiple lines and vec of blocks.
    let mut blocks = Vec::new();
    let mut buffer = String::new();
    // Remember if currently in LaTeX/Code block.
    let mut in_latex = false;
    let mut in_code = false;
    for line in note.lines() {
        // recognize type
        let btype = MdLineType::recognize_line(line);
        // convert to block
        match btype {
            // Text (non-special lines) is always appended to the buffer.
            MdLineType::Text => buffer.push_str(line),
            // New line simply writes this buffer as a block
            MdLineType::Newline => {
                if let Some(block) = MdBlock::extract_text(&mut buffer) {
                    blocks.push(block);
                    buffer.clear();
                }
            }
            // Heading also writes to a block if there was one, and then adds its own block of just this line.
            MdLineType::Heading => {
                if let Some(block) = MdBlock::extract_text(&buffer) {
                    blocks.push(block);
                    buffer.clear();
                }
                in_latex = false;
                in_code = false;
                if let Some(block) = MdBlock::extract_heading(&line) {
                    blocks.push(block);
                    buffer.clear();
                }
            }
            // LaTeX line just remembers it is latex, if it is an ending line writes to the buffer while noting that
            MdLineType::LaTeX => {
                if in_latex {
                    if let Some(block) = MdBlock::extract_latex(&buffer) {
                        blocks.push(block);
                        buffer.clear();
                    }
                }
                in_latex = !in_latex;
            }
            // Code does the same as LaTeX
            MdLineType::Code => {
                if in_code {
                    if let Some(block) = MdBlock::extract_code(&buffer) {
                        blocks.push(block);
                        buffer.clear();
                    }
                }
                in_code = !in_code;
            }
        }
    }

    blocks
}
