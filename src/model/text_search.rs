//! Literal, nonoverlapping matches retain both UTF-8 edit ranges and character selections.
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextMatch {
    pub bytes: Range<usize>,
    pub characters: Range<usize>,
}

#[must_use]
pub fn find_matches(text: &str, query: &str, match_case: bool) -> Vec<TextMatch> {
    use caseless::Caseless as _;

    if query.is_empty() {
        return Vec::new();
    }
    let mut searchable = String::with_capacity(text.len());
    let mut boundaries = Vec::new();
    for (character, (byte, value)) in text.char_indices().enumerate() {
        boundaries.push((searchable.len(), byte, character));
        if match_case {
            searchable.push(value);
        } else {
            searchable.extend(std::iter::once(value).default_case_fold());
        }
    }
    boundaries.push((searchable.len(), text.len(), boundaries.len()));
    let query = if match_case {
        query.to_owned()
    } else {
        caseless::default_case_fold_str(query)
    };
    let mut matches = Vec::new();
    let mut offset = 0;
    while let Some(relative) = searchable[offset..].find(&query) {
        let start = offset + relative;
        let end = start + query.len();
        match (
            boundaries.binary_search_by_key(&start, |entry| entry.0),
            boundaries.binary_search_by_key(&end, |entry| entry.0),
        ) {
            (Ok(first), Ok(last)) => {
                matches.push(TextMatch {
                    bytes: boundaries[first].1..boundaries[last].1,
                    characters: boundaries[first].2..boundaries[last].2,
                });
                offset = end;
            }
            _ => {
                // Skip partial expansions without hiding a later, overlapping complete match.
                offset = boundaries[boundaries.partition_point(|entry| entry.0 <= start)].0;
            }
        }
    }
    matches
}
