//! Persist tab placement before updating the visible strip, keeping the active draft intact.
use super::Mode;
use super::QuickNoteApp;
use crate::i18n;
use crate::storage::Result;

impl QuickNoteApp {
    pub(super) fn finish_tab_drag(&mut self) {
        if self.search_open {
            self.header.focus_search = true;
        } else if self.mode == Mode::Edit && !self.settings_open() {
            self.focus_editor = true;
        }
    }

    pub(super) fn reorder_tab(&mut self, id: &str, index: usize) -> Result<()> {
        let Some(previous) = self.tabs.iter().position(|note| note.id == id) else {
            return Err(i18n::message(|m| &m.error_note_missing).into());
        };
        let index = index.min(self.tabs.len() - 1);
        self.store
            .move_tab(id, index)
            .map_err(|error| i18n::message(|m| &m.error_tab_order).cause("error", error))?;
        if previous != index {
            let moved = self.tabs.remove(previous);
            self.tabs.insert(index, moved);
        }
        Ok(())
    }

    pub(super) fn move_active_tab(&mut self, right: bool) {
        let Some(previous) = self.tabs.iter().position(|note| note.id == self.note.id) else {
            return;
        };
        let index = if right {
            (previous + 1).min(self.tabs.len() - 1)
        } else {
            previous.saturating_sub(1)
        };
        let id = self.note.id.clone();
        if let Err(error) = self.reorder_tab(&id, index) {
            self.error = Some(error);
        } else if previous != index {
            self.header.reveal_active = true;
        }
    }
}
