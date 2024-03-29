mod token;
use itertools::Itertools;
pub use token::MdToken;

pub fn parse_note(note: &str) -> Vec<MdToken> {
    match_stuff(
        note,
        &regex::Regex::new(r"[\#]+\s.*\n").expect("Static regex ill-formed."),
        &|substr| {
            MdToken::new(
                token::MdTokenType::Heading(1),
                &substr.trim_start_matches(&['#', ' ']),
            )
        },
    )
}

pub fn match_stuff<F>(content: &str, regex: &regex::Regex, converter: &F) -> Vec<MdToken>
where
    F: Fn(&str) -> MdToken,
{
    // if content.len() == 0 {
    //     return Vec::new();
    // }

    std::iter::once((0, 0))
        .chain(
            regex
                .find_iter(content)
                .map(|thematch| (thematch.start(), thematch.end())),
        )
        .chain(std::iter::once((content.len(), content.len())))
        .tuple_windows()
        .map(|((_a_start, a_end), (b_start, b_end))| {
            // let mut v = match_stuff(&content[a_end..b_start], regex, converter);
            let mut v = Vec::with_capacity(2);
            if a_end != b_start {
                v.push(MdToken::new(
                    token::MdTokenType::Text,
                    &content[a_end..b_start],
                ));
            }

            if b_start != b_end {
                v.push(MdToken::new(token::MdTokenType::LineBreak, ""));
                v.push(converter(&content[b_start..b_end]));
                v.push(MdToken::new(token::MdTokenType::LineBreak, ""));
            }
            v
        })
        .flatten()
        .collect()
}
