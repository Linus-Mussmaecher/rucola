mod md_block;
pub use md_block::MdBlock;
pub use md_block::MdBlockType;

pub fn parse_note(note: &str) -> Vec<MdBlock> {
    // Recognize lines
    let lines = note
        .lines()
        .map(|line| (MdBlockType::recognize_line(line), line))
        .collect::<Vec<_>>();

    // Turn it into blocks
    let mut blocks = Vec::new();
    let mut buffer = String::new();
    let mut in_latex = false;
    let mut in_code = false;
    for (btype, line) in lines.into_iter() {
        match btype {
            // Text (non-special lines) is always appended to the buffer.
            MdBlockType::Text => buffer.push_str(line),
            // New line simply writes this buffer as a block
            MdBlockType::Newline => {
                write_block(&mut buffer, &mut blocks, MdBlockType::Text);
            }
            // Heading also writes to a block if there was one, and then adds its own block of just this line.
            MdBlockType::Heading => {
                write_block(&mut buffer, &mut blocks, MdBlockType::Text);
                in_latex = false;
                in_code = false;
                write_block(&mut line.to_owned(), &mut blocks, MdBlockType::Heading);
            }
            // LaTeX line just remembers it is latex, if it is an ending line writes to the buffer while noting that
            MdBlockType::LaTeX => {
                if in_latex {
                    write_block(&mut buffer, &mut blocks, MdBlockType::LaTeX);
                }
                in_latex = !in_latex;
            }
            // Code does the same as LaTeX
            MdBlockType::Code => {
                if in_code {
                    write_block(&mut buffer, &mut blocks, MdBlockType::Code);
                }
                in_code = !in_code;
            }
        }
    }

    blocks
}

fn write_block(buffer: &mut String, blocks: &mut Vec<MdBlock>, mdbtype: MdBlockType) {
    if !buffer.is_empty() {
        blocks.push(MdBlock::new(mdbtype, buffer.clone()));
        buffer.clear();
    }
}
