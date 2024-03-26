use crate::data::Note;

mod paragraph;
pub use paragraph::Paragraph;

pub fn parse_note(note: &str) -> Vec<Paragraph> {
    Paragraph::parse_paragraph(note).collect()
}
