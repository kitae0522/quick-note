use super::CursorLocation;
use super::Selection;

#[derive(Clone, Debug)]
pub struct Lines {
    pub starts: Vec<usize>,
    characters: Vec<usize>,
    utf16: Vec<usize>,
    character_count: usize,
    utf16_count: usize,
}

impl Lines {
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        let mut characters = vec![0];
        let mut utf16 = vec![0];
        let mut character_count = 0;
        let mut utf16_count = 0;
        for (byte, character) in text.char_indices() {
            character_count += 1;
            utf16_count += character.len_utf16();
            if character == '\n'
                || (character == '\r' && text.as_bytes().get(byte + 1) != Some(&b'\n'))
            {
                starts.push(byte + 1);
                characters.push(character_count);
                utf16.push(utf16_count);
            }
        }
        Self {
            starts,
            characters,
            utf16,
            character_count,
            utf16_count,
        }
    }

    fn character_line(&self, character: usize) -> usize {
        self.characters
            .partition_point(|start| *start <= character)
            .saturating_sub(1)
    }

    fn to_utf16(&self, text: &str, character: usize) -> usize {
        let character = character.min(self.character_count);
        let line = self.character_line(character);
        self.utf16[line]
            + text[self.starts[line]..]
                .chars()
                .take(character - self.characters[line])
                .map(char::len_utf16)
                .sum::<usize>()
    }

    fn to_characters(&self, text: &str, offset: usize) -> usize {
        let offset = offset.min(self.utf16_count);
        let line = self
            .utf16
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        let mut units = self.utf16[line];
        self.characters[line]
            + text[self.starts[line]..]
                .chars()
                .take_while(|character| {
                    units += character.len_utf16();
                    units <= offset
                })
                .count()
    }

    /// Converts egui's character range to the existing UTF-16 persisted cursor format.
    #[must_use]
    pub fn selection(&self, text: &str, start: usize, end: usize) -> Selection {
        let location = self.to_utf16(text, start.min(end));
        Selection {
            location,
            length: self.to_utf16(text, start.max(end)) - location,
        }
    }

    #[must_use]
    pub fn char_range(&self, text: &str, selection: &Selection) -> (usize, usize) {
        (
            self.to_characters(text, selection.location),
            self.to_characters(text, selection.location.saturating_add(selection.length)),
        )
    }

    #[must_use]
    pub fn location(&self, text: &str, character: usize) -> CursorLocation {
        use unicode_segmentation::UnicodeSegmentation;
        let character = character.min(self.character_count);
        let line = self.character_line(character);
        let byte = self.starts[line]
            + text[self.starts[line]..]
                .char_indices()
                .nth(character - self.characters[line])
                .map_or(text.len() - self.starts[line], |(byte, _)| byte);
        let column = text[self.starts[line]..byte].graphemes(true).count();
        CursorLocation {
            line: line + 1,
            column: column + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_lines_count_crlf_and_trailing_empty_line() {
        let text = "가💡\r\ne\u{301}\nlast\r";
        let lines = Lines::new(text);
        assert_eq!(lines.starts.len(), 4);
        assert_eq!(
            lines.location(text, 2),
            CursorLocation { line: 1, column: 3 }
        );
        assert_eq!(
            lines.location(text, 6),
            CursorLocation { line: 2, column: 2 }
        );
        assert_eq!(
            lines.location(text, text.chars().count()),
            CursorLocation { line: 4, column: 1 }
        );
    }

    #[test]
    fn indexed_cursor_conversions_match_persisted_unicode_ranges() {
        for text in [
            "",
            "ascii",
            "가💡\r\ne\u{301}\nlast\r",
            "👩‍💻\n한글\n",
            "\r\n\r\n",
        ] {
            let lines = Lines::new(text);
            for start in 0..=text.chars().count() + 2 {
                for end in 0..=text.chars().count() + 2 {
                    assert_eq!(
                        lines.selection(text, start, end),
                        Selection::from_char_range(text, start, end)
                    );
                }
            }
            for location in (0..=text.encode_utf16().count() + 2).chain([usize::MAX]) {
                for length in [0, 1, 3, usize::MAX] {
                    let selection = Selection { location, length };
                    assert_eq!(
                        lines.char_range(text, &selection),
                        selection.char_range(text)
                    );
                }
            }
        }
    }
}
