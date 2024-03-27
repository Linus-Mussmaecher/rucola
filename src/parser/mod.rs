mod paragraph;
pub use paragraph::Paragraph;

pub fn parse_note(note: &str) -> Vec<Paragraph> {
    note.split("\n\n")
        .map(|par| Paragraph::parse_paragraph(par))
        .flatten()
        .collect()
}
