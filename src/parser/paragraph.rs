use ratatui::text::Line;
use regex::Regex;

use crate::ui;

enum ParagraphType {
    HEADING,
    TEXT,
}

pub struct Paragraph {
    content: String,
    ptype: ParagraphType,
}

impl Paragraph {
    pub fn parse_paragraph(paragraph: &str) -> Vec<Paragraph> {
        let heading_reg = Regex::new(r"^(\#)+\s").expect("nope");

        // Check if it starts with a header
        if heading_reg.is_match(paragraph) {
            // Separate the header

            if let Some((header, rest)) = paragraph.split_once("\n") {
                vec![
                    Self {
                        content: header.replace(r"#", ""),
                        ptype: ParagraphType::HEADING,
                    },
                    Self {
                        content: rest.to_string(),
                        ptype: ParagraphType::TEXT,
                    },
                ]
            } else {
                vec![]
            }
        } else {
            // must be just text
            vec![Self {
                content: paragraph.to_string(),
                ptype: ParagraphType::TEXT,
            }]
        }
    }

    pub fn to_widget<'a>(&'a self, styles: ui::MdStyles) -> Line<'a> {
        match self.ptype {
            ParagraphType::HEADING => Line::from(self.content.as_str()).style(styles.heading),
            ParagraphType::TEXT => Line::from(self.content.as_str()),
        }
    }
}
