use fuzzy_matcher::FuzzyMatcher;
/// Describes a way to filter notes by their contained tags and/or title
#[derive(Debug, Default, Clone)]
pub struct Filter {
    /// Wether or not all specified tags must be contained in the note in order to match the filter, or only any (=at least one) of them.
    pub any: bool,
    /// The tags to include and exclude by, hash included.
    pub tags: Vec<(String, bool)>,
    /// The links to look for or exclude, already converted to ids.
    pub links: Vec<(String, bool)>,
    /// The backlinks to look for or exclude, already converted to ids.
    pub blinks: Vec<(String, bool)>,
    /// The words to search the note title for. Will be fuzzy matched with the note title.
    pub title: String,
    /// Everything to be searched for in the full text of the notes, in lowercase.
    pub full_text: Option<String>,
}

impl Filter {
    pub fn new(filter_string: &str, any: bool) -> Self {
        let mut tags = Vec::new();
        let mut links = Vec::new();
        let mut blinks = Vec::new();
        let mut title = String::new();

        let (filters, full_text) = filter_string
            .split_once('|')
            .map(|(filters, rest)| (filters, Some(rest.to_lowercase())))
            .unwrap_or((filter_string, None));

        // Go through words
        for word in filters.split_whitespace() {
            if word.starts_with("!#") {
                tags.push((word.trim_start_matches('!').to_string(), false));
                continue;
            }
            if word.starts_with('#') {
                tags.push((word.to_string(), true));
                continue;
            }
            if word.starts_with("!>") {
                links.push((
                    super::name_to_id(word.trim_start_matches("!>")).to_string(),
                    false,
                ));
                continue;
            }
            if word.starts_with('>') {
                links.push((
                    super::name_to_id(word.trim_start_matches('>')).to_string(),
                    true,
                ));
                continue;
            }
            if word.starts_with("!<") {
                blinks.push((
                    super::name_to_id(word.trim_start_matches("!<")).to_string(),
                    false,
                ));
                continue;
            }
            if word.starts_with('<') {
                blinks.push((
                    super::name_to_id(word.trim_start_matches('<')).to_string(),
                    true,
                ));
                continue;
            }
            // if nothing else fits
            title.push_str(word);
        }

        // check for any or all tags
        Self {
            any,
            tags,
            links,
            blinks,
            title,
            full_text,
        }
    }

    pub fn apply(&self, note: &super::Note, index: &super::NoteIndex) -> Option<i64> {
        // === === TAGS === ===

        let mut any = false;
        let mut all = true;
        for (tag, included) in self.tags.iter() {
            if note
                // go over all tags
                .tags
                .iter()
                // split each tag into..
                .flat_map(|tag| {
                    // an iterator of substring starting at 0 and going to every appearance to /
                    tag.match_indices('/')
                        .map(|(index, _match)| &tag[0..index])
                        // and appended just a substring that is the whole tag
                        .chain(std::iter::once(tag.as_str()))
                    // flatten this so we have just an iterator over (sub)strs
                })
                // check if any of these substring is the searched tag or, in case of a multi-word tag, the tag with appropriate replacements.
                .any(|subtag| subtag == tag || subtag == tag.replace("-", " "))
            // now compare this to our expectation
            //  - inclusion: We _want_ one of them to be equal
            //  - exclusion: We _dont_ want one of them to be equal
             == *included
            {
                // this did match our expectation (one of them is equal in case of inclusion or none of them is equal in case of exclusion)
                // so at least one condition (this one) is true
                any = true;
            } else {
                // this did not match our expectation (none of them is equal in case of inclusion or one of them is equal in case of exclusion)
                // so not all conditions can be true
                all = false;
            }
        }

        // === === LINKS === ===

        // go through all links
        for (link, included) in self.links.iter() {
            // if the links is contained and we want it to be contained or not contained and we want it to be not contained
            if note.links.contains(link) == *included {
                // at least one condition (this one) is true
                any = true;
            } else {
                // else, at least one condition is false, so not all of them are true
                all = false;
            }
        }

        // go through all backlinks
        for (blink, included) in self.blinks.iter() {
            // check if the note with the blink-ID links to the main one passed to this function
            let exists_and_contains = if let Some(other_note) = index.inner.get(blink) {
                other_note.links.contains(&super::name_to_id(&note.name))
            } else {
                false
            };

            // if the backlink exists and we want that, set any/all as above
            if exists_and_contains == *included {
                any = true;
            } else {
                all = false;
            }
        }

        if let Some(text) = &self.full_text {
            if std::fs::read_to_string(&note.path)
                .map(|content| content.to_lowercase().contains(text))
                .unwrap_or(false)
            {
                any = true;
            } else {
                all = false;
            }
        }

        let fuz_match = if self.title.is_empty() {
            None
        } else {
            let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
            let fuzzy_match = matcher.fuzzy_match(&note.display_name, &self.title);
            if fuzzy_match.is_some() {
                any = true;
            } else {
                all = false;
            }
            fuzzy_match
        };
        // if all conditions are empty, return match score (only title search)
        if self.tags.is_empty() && self.links.is_empty() && self.blinks.is_empty() && self.full_text.is_none() && self.title.is_empty()  ||
            // also return match score if the required amount of conditions are fulfilled
            (!self.any && all || self.any && any)
        {
            fuz_match.or(Some(0))
        } else {
            // else, an exclusion criterion was triggered
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data, io};

    #[test]
    fn test_filter() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        let linux = index.inner.get("linux").unwrap();
        let win = index.inner.get("windows").unwrap();
        let osx = index.inner.get("osx").unwrap();

        // === Filter 1 ===

        let filter1 = Filter {
            any: false,
            tags: vec![("#os".to_string(), true), ("#os/win".to_string(), false)],
            links: vec![],
            blinks: vec![],
            title: String::new(),
            full_text: None,
        };

        assert!(filter1.apply(linux, &index).is_some());
        assert!(filter1.apply(osx, &index).is_some());
        assert!(filter1.apply(win, &index).is_none());
    }

    #[test]
    fn test_filter_from_string() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        // === Filter 2 ===

        let filter2 = Filter::new("!#lietheo #diffgeo >Manifold !>atlas", false);

        assert_eq!(
            filter2.tags,
            vec![
                ("#lietheo".to_string(), false),
                ("#diffgeo".to_string(), true)
            ]
        );
        assert_eq!(
            filter2.links,
            vec![("manifold".to_string(), true), ("atlas".to_string(), false)]
        );
        assert_eq!(filter2.title, "");

        let liegroup = index.inner.get("lie-group").unwrap();
        let chart = index.inner.get("chart").unwrap();
        let manifold = index.inner.get("manifold").unwrap();
        let smoothmap = index.inner.get("smooth-map").unwrap();
        let topology = index.inner.get("topology").unwrap();

        assert!(filter2.apply(liegroup, &index).is_none());
        assert!(filter2.apply(chart, &index).is_some());
        assert!(filter2.apply(manifold, &index).is_none());
        assert!(filter2.apply(smoothmap, &index).is_none());
        assert!(filter2.apply(topology, &index).is_none());
    }

    #[test]
    fn test_filter_from_string_all() {
        // === Filter 3 ===
        let filter3 = Filter::new(
            "!#topology #os >TopologY !>Smooth-mAp <atlas !<linux |equivalent",
            false,
        );

        assert_eq!(
            filter3.tags,
            vec![("#topology".to_string(), false), ("#os".to_string(), true)]
        );
        assert_eq!(
            filter3.links,
            vec![
                ("topology".to_string(), true),
                ("smooth-map".to_string(), false)
            ]
        );
        assert_eq!(
            filter3.blinks,
            vec![("atlas".to_string(), true), ("linux".to_string(), false)]
        );
        assert_eq!(filter3.title, "");

        assert_eq!(filter3.full_text, Some(String::from("equivalent")));
    }

    #[test]
    fn test_filter_from_string_case_insensitive() {
        // === Filter 4 ===
        let filter4 = Filter::new("<aTlas >Smooth-mAp", true);

        assert_eq!(filter4.tags, vec![]);
        assert_eq!(filter4.links, vec![("smooth-map".to_string(), true),]);
        assert_eq!(filter4.blinks, vec![("atlas".to_string(), true)]);
        assert_eq!(filter4.title, "");
    }

    #[test]
    fn test_filter_from_string_multi_word_tags() {
        let config = crate::Config::default();
        let tracker = io::FileTracker::new(&config, std::path::PathBuf::from("./tests")).unwrap();
        let builder = io::HtmlBuilder::new(&config, std::path::PathBuf::from("./tests"));
        let index = data::NoteIndex::new(tracker, builder).0;

        assert_eq!(index.inner.len(), 12);

        // === Filter 2 ===

        let filter2 = Filter::new("#funny-abbreviations", false);

        assert_eq!(
            filter2.tags,
            vec![("#funny-abbreviations".to_string(), true),]
        );

        assert_eq!(filter2.links, vec![]);

        assert_eq!(filter2.title, "");

        let yamlformat = index.inner.get("note25").unwrap();
        let chart = index.inner.get("chart").unwrap();

        assert!(filter2.apply(yamlformat, &index).is_some());
        assert!(filter2.apply(chart, &index).is_none());
    }
}
