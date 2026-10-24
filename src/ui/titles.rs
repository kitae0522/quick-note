use crate::i18n::Messages;
use crate::model::NoteSummary;

/// Only generated blank-note titles are translated; user-authored titles stay intact.
#[must_use]
pub fn note_title<'a>(note: &'a NoteSummary, text: &'a Messages) -> &'a str {
    if note.is_blank {
        &text.note_untitled
    } else {
        &note.title
    }
}
