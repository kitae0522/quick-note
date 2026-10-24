//! Opt-in, ordered text replacements for editor-specific commands.
use crate::Event;
use crate::TextBuffer;
use crate::text::CCursorRange;
use core::ops::Range;
use epaint::text::CharIndex;

/// An atomic replacement and resulting selection, expressed in character indices.
/// An empty range and empty text move only the cursor, without changing the document.
pub struct TextEditMutation {
    pub range: Range<CharIndex>,
    pub text: String,
    pub cursor_range: CCursorRange,
}

/// Called in event order with the current selection, outside active IME composition.
/// Return `None` to leave the event to TextEdit's standard handling.
pub type TextEditInputHandler<'a> =
    dyn FnMut(&Event, &dyn TextBuffer, CCursorRange) -> Option<TextEditMutation> + 'a;
