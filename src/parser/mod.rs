mod token;
use itertools::Itertools;
pub use token::MdToken;

mod token_parser;
use token_parser::TokenParser;

pub fn parse_note(note: &str) -> Vec<MdToken> {
    parse_recursively(
        note,
        &[
            TokenParser::create_line_break_parser(),
            TokenParser::create_headings_parser(),
            TokenParser::create_tag_parser(),
            TokenParser::create_double_star_parser(),
            TokenParser::create_star_parser(),
            TokenParser::create_underscore_parser(),
            TokenParser::create_text_parser(),
        ],
    )
}

pub fn parse_recursively(content: &str, token_parsers: &[TokenParser]) -> Vec<MdToken> {
    if token_parsers.is_empty() {
        return vec![];
    }

    std::iter::once((0, 0))
        .chain(
            token_parsers[0]
                .get_regex()
                .find_iter(content)
                .map(|thematch| (thematch.start(), thematch.end())),
        )
        .chain(std::iter::once((content.len(), content.len())))
        .tuple_windows()
        .flat_map(|((_a_start, a_end), (b_start, b_end))| {
            // let mut v = match_stuff(&content[a_end..b_start], regex, converter);
            let mut v = if a_end != b_start {
                parse_recursively(&content[a_end..b_start], &token_parsers[1..])
            } else {
                Vec::with_capacity(1)
            };

            if b_start != b_end {
                if let Some(mdtoken) = token_parsers[0].convert(&content[b_start..b_end]) {
                    v.push(mdtoken);
                }
            }
            v
        })
        .collect()
}
