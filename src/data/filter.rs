use fuzzy_matcher::FuzzyMatcher;

/// Describes a way to filter notes by their contained tags and/or title
#[derive(Debug, Default, Clone)]
pub struct Filter {
    /// Wether or not all specified tags must be contained in the note in order to match the filter, or only any (=at least one) of them.
    pub all: bool,
    /// The tags to require by, hash included.
    pub include_tags: Vec<String>,
    /// The tags to avoid by, hash included.
    pub exclude_tags: Vec<String>,
    /// The words to search the note title for. Will be fuzzy matched with the note title.
    pub title: String,
}

impl Filter {
    pub fn new(filter_string: &str, all: bool) -> Self {
        let mut exclude_tags = Vec::new();
        let mut include_tags = Vec::new();

        let mut title = String::new();

        // Go through words
        for word in filter_string.split_whitespace() {
            if word.starts_with("!#") {
                exclude_tags.push(word.trim_start_matches("!").to_string());
                continue;
            }
            if word.starts_with('#') {
                // words with a hash count as a tag
                include_tags.push(word.to_string());
                continue;
            }
            // if nothing else fits
            title.push_str(word);
        }

        // check for any or all tags
        Self {
            all,
            exclude_tags,
            include_tags,
            title,
        }
    }

    pub fn apply(&self, note: &super::Note) -> Option<i64> {
        let mut any = false;
        let mut all = true;
        for include_tag in self.include_tags.iter() {
            if !note.tags.contains(include_tag) {
                all = false;
            } else {
                any = true;
            }
        }

        if !self.include_tags.is_empty() && ((self.all && !all) || (!self.all && !any)) {
            return None;
        }

        any = false;
        all = true;
        for exclude_tag in self.exclude_tags.iter() {
            if note.tags.contains(exclude_tag) {
                all = false;
            } else {
                any = true;
            }
        }

        if !self.exclude_tags.is_empty() && ((self.all && !all) || (!self.all && !any)) {
            return None;
        }

        // If nothing has triggerd an exclusion criterion, return the fuzzy match score
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        return matcher.fuzzy_match(&note.name, &self.title);
    }
}
