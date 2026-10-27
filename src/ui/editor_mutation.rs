//! Programmatic replacements share TextEdit's undo history and persisted cursor units.
use crate::model::Note;
use crate::model::Selection;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::text::CCursor;
use eframe::egui::text::CCursorRange;
use eframe::egui::text_edit::TextEditState;

pub(crate) fn replace(context: &Context, note: &mut Note, body: String, caret: usize) -> bool {
    if note.body == body {
        return false;
    }
    let id = Id::unique(("note-editor", &note.id));
    let mut state = TextEditState::load(context, id).unwrap_or_default();
    let (start, end) = note.selection.char_range(&note.body);
    let before = state
        .cursor
        .char_range()
        .unwrap_or_else(|| CCursorRange::two(CCursor::new(start), CCursor::new(end)));
    let after = CCursorRange::one(CCursor::new(caret));
    let mut undoer = state.undoer();
    undoer.add_undo(&(before, note.body.clone()));
    // TextEdit feeds the final cursor after layout; an extra checkpoint adds cursor-only undos.
    undoer.feed_state(context.input(|input| input.time), &(after, body.clone()));
    state.set_undoer(undoer);
    state.cursor.set_char_range(Some(after));
    state.store(context, id);
    note.body = body;
    note.selection = Selection::from_char_range(&note.body, caret, caret);
    true
}
