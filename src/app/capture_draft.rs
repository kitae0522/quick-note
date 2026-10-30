//! Fresh captures stay in memory until their first text is durably committed.
use super::QuickNoteApp;
use crate::i18n;
use crate::model::Note;
use crate::model::Selection;
use crate::storage;
use crate::storage::Result;
use crate::ui::FocusTarget;
use eframe::egui::Context;

impl QuickNoteApp {
    pub(super) fn new_capture(&mut self, context: &Context) -> Result<()> {
        if self.dirty && !self.persist() {
            return Err(i18n::message(|m| &m.error_save_before_new).into());
        }
        if self.capture.draft_origin.is_none() {
            self.display_capture_draft(storage::draft_note());
        }
        self.toggle_capture(context, false);
        Ok(())
    }

    pub(super) fn display_capture_draft(&mut self, note: Note) {
        self.discard_capture_draft();
        let previous = self.note.clone();
        self.tabs.push(note.summary());
        self.display(note, FocusTarget::Editor);
        self.capture.draft_origin = Some(previous);
    }

    pub(super) fn discard_capture_draft(&mut self) {
        if let Some(previous) = self.capture.draft_origin.take() {
            self.tabs.retain(|tab| tab.id != self.note.id);
            self.display(previous, FocusTarget::Editor);
        }
    }

    /// Captures supplied text without modifying the active note, its cursor or open tabs.
    pub fn capture_text(&mut self, text: String) -> Result<String> {
        if text.is_empty() {
            return Err(i18n::message(|m| &m.capture.clipboard_empty).into());
        }
        if self.dirty && !self.persist() {
            return Err(i18n::message(|m| &m.error_save_before_new).into());
        }
        let mut note = storage::draft_note();
        note.body = text;
        note.selection = Selection::from_char_range(
            &note.body,
            note.body.chars().count(),
            note.body.chars().count(),
        );
        match self.store.capture_note(&note, false) {
            Ok(updated_at) => {
                note.updated_at = updated_at;
                if let Some(cache) = &mut self.search_cache {
                    cache.upsert(&note);
                }
                self.search_query = None;
                self.capture.recent_dirty = true;
                Ok(note.title())
            }
            Err(error) => {
                self.display_capture_draft(note);
                self.record_save_failure(error);
                Err(i18n::message(|m| &m.error_save_before_new).into())
            }
        }
    }
}
