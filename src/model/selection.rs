use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    // UTF-16 offsets are the persisted cursor format, including existing notes.
    pub location: usize,
    pub length: usize,
}

impl Selection {
    #[must_use]
    pub fn clamped(&self, text: &str) -> Self {
        let count = text.encode_utf16().count();
        let location = self.location.min(count);
        Self {
            location,
            length: self.length.min(count - location),
        }
    }

    #[must_use]
    pub fn from_char_range(text: &str, start: usize, end: usize) -> Self {
        let location = text.chars().take(start.min(end)).map(char::len_utf16).sum();
        let end: usize = text.chars().take(start.max(end)).map(char::len_utf16).sum();
        Self {
            location,
            length: end - location,
        }
    }

    #[must_use]
    pub fn char_range(&self, text: &str) -> (usize, usize) {
        let selection = self.clamped(text);
        let to_chars = |offset: usize| {
            let mut units = 0;
            text.chars()
                .take_while(|character| {
                    units += character.len_utf16();
                    units <= offset
                })
                .count()
        };
        (
            to_chars(selection.location),
            to_chars(selection.location + selection.length),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_cursor_round_trip_preserves_persisted_offsets() {
        let text = "가💡e\u{301}z";
        let selected = Selection::from_char_range(text, 1, 4);
        assert_eq!(
            selected,
            Selection {
                location: 1,
                length: 4
            }
        );
        assert_eq!(selected.char_range(text), (1, 4));
        assert_eq!(
            Selection {
                location: usize::MAX,
                length: usize::MAX
            }
            .char_range(text),
            (5, 5)
        );
    }
}
