//! Apply paired delimiters at the widget's current cursor, preserving ordered input and undo.
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::TextBuffer;
use eframe::egui::TextEditMutation;
use eframe::egui::text::CCursorRange;

fn closing(opening: u8) -> Option<&'static str> {
    match opening {
        b'(' => Some(")"),
        b'[' => Some("]"),
        b'{' => Some("}"),
        _ => None,
    }
}

pub(super) fn input(
    event: &Event,
    text: &dyn TextBuffer,
    cursor: CCursorRange,
) -> Option<TextEditMutation> {
    let range = cursor.as_sorted_char_range();
    match event {
        Event::Text(value) if value.len() == 1 => {
            let character = value.as_bytes()[0];
            if let Some(close) = closing(character) {
                let inserted = format!("{value}{}{close}", text.char_range(range.clone()));
                let mut selection = cursor;
                selection.primary.index += 1;
                selection.secondary.index += 1;
                return Some(TextEditMutation {
                    range,
                    text: inserted,
                    cursor_range: selection,
                });
            }
            if cursor.is_empty()
                && matches!(character, b')' | b']' | b'}')
                && text.char_range(range.start..range.start + 1) == value
            {
                let mut caret = cursor.primary;
                caret.index += 1;
                return Some(TextEditMutation {
                    range,
                    text: String::new(),
                    cursor_range: CCursorRange::one(caret),
                });
            }
            None
        }
        Event::Key {
            key: Key::Backspace,
            pressed: true,
            modifiers,
            ..
        } if modifiers.is_none() && cursor.is_empty() && range.start.0 > 0 => {
            let pair = text.char_range(range.start - 1..range.start + 1).as_bytes();
            if pair.len() == 2
                && closing(pair[0]).is_some_and(|close| close.as_bytes()[0] == pair[1])
            {
                let mut caret = cursor.primary;
                caret.index -= 1;
                Some(TextEditMutation {
                    range: range.start - 1..range.start + 1,
                    text: String::new(),
                    cursor_range: CCursorRange::one(caret),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}
