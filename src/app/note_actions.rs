//! Note creation, activation and deletion policy.
use super::Mode;
use super::QuickNoteApp;
use crate::i18n;
use crate::model::Note;
use crate::storage::Result;
use crate::ui::FocusTarget;

impl QuickNoteApp {
    pub fn select_note(&mut self, id: &str, focus: FocusTarget) -> Result<()> {
        if self.dirty && !self.persist() {
            return Err(i18n::message(|m| &m.error_save_before_switch).into());
        }
        let note = self.store.activate(id)?;
        if !self.tabs.iter().any(|tab| tab.id == id) {
            self.tabs = self.store.summaries()?;
        }
        self.display(note, focus);
        Ok(())
    }

    pub fn new_note(&mut self) -> Result<()> {
        if self.dirty && !self.persist() {
            return Err(i18n::message(|m| &m.error_save_before_new).into());
        }
        let note = self.store.create_note()?;
        self.tabs.push(note.summary());
        self.display(note, FocusTarget::Editor);
        if let Some(cache) = &mut self.search_cache {
            cache.upsert(&self.note);
        }
        self.search_previews.clear();
        self.search_query = None;
        Ok(())
    }

    pub(super) fn display(&mut self, note: Note, focus: FocusTarget) {
        if self.capture.draft_origin.take().is_some() {
            self.tabs.retain(|tab| tab.id != self.note.id);
        }
        self.capture.recent_dirty = true;
        self.document_find.close();
        self.note = note;
        self.markdown_cache = None;
        self.composing = false;
        self.restore_cursor = true;
        self.focus_editor = focus == FocusTarget::Editor;
        self.mode = Mode::Edit;
        self.header.reveal_active = true;
        self.search_open = false;
        self.settings = None;
        self.settings_request = None;
        self.dirty = false;
        self.error = None;
    }

    /// Deletes blank notes immediately; notes with text wait for confirmation.
    pub fn request_delete(&mut self, id: &str) -> Result<()> {
        if self.tabs.iter().any(|tab| tab.id == id) {
            let blank = if id == self.note.id {
                self.note.body.trim().is_empty()
            } else {
                self.store
                    .note(id)?
                    .is_some_and(|note| note.body.trim().is_empty())
            };
            self.pending_delete = Some(id.into());
            self.modal_new = true;
            self.search_open = false;
            self.settings = None;
            self.settings_request = None;
            if blank {
                self.confirm_delete()?;
            }
        }
        Ok(())
    }

    pub fn confirm_delete(&mut self) -> Result<()> {
        if self.dirty && !self.persist() {
            return Err(i18n::message(|m| &m.error_save_before_delete).into());
        }
        let id = self
            .pending_delete
            .clone()
            .ok_or(i18n::message(|m| &m.error_delete_selection))?;
        if self.capture.draft_origin.is_some() && id == self.note.id {
            self.pending_delete = None;
            self.discard_capture_draft();
            return Ok(());
        }
        let next = self.store.delete_confirmed(&id)?;
        self.clear_recovery(&id);
        self.tabs = self.store.summaries()?;
        self.pending_delete = None;
        if let Some(cache) = &mut self.search_cache {
            cache.remove(&id);
            cache.upsert(&next);
        }
        self.search_previews.clear();
        self.search_query = None;
        self.display(next, FocusTarget::Editor);
        Ok(())
    }

    pub fn cancel_delete(&mut self) {
        self.pending_delete = None;
        self.focus_editor = self.previous_focus.is_none() && self.mode == Mode::Edit;
    }
}
