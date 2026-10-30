//! Short UTF-8 excerpts retain the original selection even when case folding expands text.
use super::Selection;
use super::find_matches;
use std::ops::Range;

#[derive(Clone, Debug)]
pub(crate) struct SearchPreview {
    pub text: String,
    pub highlight: Option<Range<usize>>,
    pub selection: Option<Selection>,
}

impl SearchPreview {
    pub fn new(body: &str, query: &str) -> Self {
        let found = find_matches(body, query, false).into_iter().next();
        let (start, end) = found
            .as_ref()
            .map_or((0, 0), |found| (found.bytes.start, found.bytes.end));
        let from = body[..start]
            .char_indices()
            .rev()
            .nth(35)
            .map_or(0, |(byte, _)| byte);
        let to = body[end..]
            .char_indices()
            .nth(100)
            .map_or(body.len(), |(byte, _)| end + byte);
        let mut text = if from > 0 {
            "…".to_owned()
        } else {
            String::new()
        };
        let prefix = text.len();
        text.push_str(&body[from..to].replace(['\n', '\r', '\t'], " "));
        if to < body.len() {
            text.push('…');
        }
        Self {
            text,
            highlight: found
                .as_ref()
                .map(|_| prefix + start - from..prefix + end - from),
            selection: found.map(|found| {
                Selection::from_char_range(body, found.characters.start, found.characters.end)
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folded_excerpts_preserve_the_exact_original_utf16_selection() {
        let body = format!(
            "{}\n한💡 Straße café\n{}",
            "앞".repeat(80),
            "끝".repeat(180)
        );
        let preview = SearchPreview::new(&body, "STRASSE");
        let range = preview.highlight.expect("full case-folded match");
        assert_eq!(&preview.text[range], "Straße");
        let (start, end) = preview
            .selection
            .expect("original cursor")
            .char_range(&body);
        assert_eq!(
            body.chars()
                .skip(start)
                .take(end - start)
                .collect::<String>(),
            "Straße"
        );
        assert!(preview.text.starts_with('…') && preview.text.ends_with('…'));
        assert!(preview.text.chars().count() < 150);
    }
}
