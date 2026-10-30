//! Tab closure and deleted-note recovery preserve the failed-save guard.
use super::QuickNoteApp;
use crate::i18n;
use crate::storage::Result;
use crate::ui::FocusTarget;
use eframe::egui::Context;

impl QuickNoteApp {
    pub fn close_tab(&mut self, id: &str) -> Result<()> {
        if self.dirty && !self.persist() {
            return Err(i18n::message(|m| &m.error_save_before_switch).into());
        }
        if self.capture.draft_origin.is_some() && id == self.note.id {
            self.discard_capture_draft();
            return Ok(());
        }
        let note = self.store.close_tab(id)?;
        self.tabs = self.store.summaries()?;
        if let Some(cache) = &mut self.search_cache {
            cache.upsert(&note);
        }
        self.search_query = None;
        if note.id != self.note.id {
            self.display(note, FocusTarget::Editor);
        }
        Ok(())
    }

    pub fn restore_deleted(&mut self, id: &str) -> Result<()> {
        if self.dirty && !self.persist() {
            return Err(i18n::message(|m| &m.error_save_before_switch).into());
        }
        let note = self.store.restore_deleted(id)?;
        if let Some(cache) = &mut self.search_cache {
            cache.upsert(&note);
        }
        self.tabs = self.store.summaries()?;
        self.search_query = None;
        self.display(note, FocusTarget::Editor);
        Ok(())
    }

    pub(super) fn open_search_result(&mut self, id: &str, context: &Context) -> Result<()> {
        if self.search_deleted {
            return self.restore_deleted(id);
        }
        let query = self.header.query.trim().to_owned();
        let selection = self
            .search_previews
            .get(id)
            .and_then(|preview| preview.selection.clone());
        self.select_note(id, FocusTarget::Editor)?;
        if !query.is_empty() {
            if let Some(selection) = selection {
                self.note.selection = selection;
            }
            self.document_find.query = query;
            self.document_find.match_case = false;
            self.document_find.open = true;
            self.document_find.refresh(&self.note);
            self.document_find.active = 0;
            self.select_find_match();
            self.document_find.focus = false;
            context.request_repaint();
        }
        Ok(())
    }
}
