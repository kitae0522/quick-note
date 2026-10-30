//! Commit edits and recover a failed database write without discarding text.
use super::QuickNoteApp;
use super::files;
use crate::i18n;
use crate::model::Note;
use crate::ui::FocusTarget;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
#[derive(Serialize, Deserialize)]
struct RecoveryDraft {
    note: Note,
    #[serde(default)]
    new_note: bool,
}

impl QuickNoteApp {
    /// Commits the current draft; failure retains editor text and attempts a recovery copy.
    /// Callers must confirm success before switching or discarding the current draft.
    #[must_use]
    pub fn persist(&mut self) -> bool {
        if self.capture.draft_origin.is_some() && self.note.body.is_empty() {
            self.clear_recovery(&self.note.id.clone());
            if self.recovery_id.as_deref() == Some(&self.note.id) {
                return false;
            }
            self.dirty = false;
            self.error = None;
            return true;
        }
        let saved = if self.capture.draft_origin.is_some() {
            self.store.capture_note(&self.note, true)
        } else {
            self.store.save(&self.note)
        };
        match saved {
            Ok(updated_at) => {
                self.capture.draft_origin = None;
                self.capture.recent_dirty = true;
                self.note.updated_at = updated_at;
                self.dirty = false;
                self.error = None;
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == self.note.id) {
                    *tab = self.note.summary();
                }
                if let Some(cache) = &mut self.search_cache {
                    cache.upsert(&self.note);
                }
                self.search_previews.remove(&self.note.id);
                self.search_query = None;
                self.clear_recovery(&self.note.id.clone());
                true
            }
            Err(error) => {
                self.record_save_failure(error);
                false
            }
        }
    }

    pub(super) fn record_save_failure(&mut self, error: i18n::Error) {
        self.dirty = true;
        let recovery = RecoveryDraft {
            note: self.note.clone(),
            new_note: self.capture.draft_origin.is_some(),
        };
        let saved = serde_json::to_vec(&recovery)
            .map_err(Into::into)
            .and_then(|bytes| files::atomic_write(&self.directory.join("recovery.json"), &bytes));
        self.error = Some(match saved {
            Ok(()) => {
                self.recovery_id = Some(self.note.id.clone());
                i18n::message(|m| &m.error_save_recovery)
                    .cause("error", error)
                    .into()
            }
            Err(recovery_error) => i18n::message(|m| &m.error_save_no_recovery)
                .cause("error", error)
                .cause("recovery_error", recovery_error)
                .into(),
        });
    }

    pub(super) fn restore_recovery(&mut self) {
        let path = self.directory.join("recovery.json");
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => {
                self.error = Some(
                    i18n::message(|m| &m.error_recovery_read)
                        .cause("error", error)
                        .into(),
                );
                return;
            }
        };
        let draft: RecoveryDraft = match serde_json::from_slice(&bytes) {
            Ok(draft) => draft,
            Err(error) => {
                self.error = Some(
                    i18n::message(|m| &m.error_recovery_decode)
                        .cause("error", error)
                        .into(),
                );
                return;
            }
        };
        self.recovery_id = Some(draft.note.id.clone());
        if draft.new_note {
            match self.store.contains_note(&draft.note.id) {
                Ok(false) => {
                    self.display_capture_draft(draft.note);
                    self.dirty = true;
                    let _saved = self.persist();
                }
                Ok(true) => self.clear_recovery(&draft.note.id),
                Err(error) => {
                    self.error = Some(
                        i18n::message(|m| &m.error_recovery_wait)
                            .cause("error", error)
                            .into(),
                    )
                }
            }
            return;
        }
        match self.store.note(&draft.note.id) {
            // Restore only against the exact saved revision, even if the wall clock changed.
            Ok(Some(stored)) if draft.note.updated_at == stored.updated_at => {
                self.display(draft.note, FocusTarget::Editor);
                self.dirty = true;
                let _saved = self.persist();
                if !self.dirty
                    && let Err(error) = self.store.set_setting("activeNote", &self.note.id)
                {
                    self.error = Some(
                        i18n::message(|m| &m.error_recovery_active)
                            .cause("error", error)
                            .into(),
                    );
                }
            }
            // A later successful save or confirmed deletion takes precedence over a stale copy.
            Ok(_) => self.clear_recovery(&draft.note.id),
            Err(error) => {
                self.error = Some(
                    i18n::message(|m| &m.error_recovery_wait)
                        .cause("error", error)
                        .into(),
                );
            }
        }
    }

    pub(super) fn clear_recovery(&mut self, id: &str) {
        if self.recovery_id.as_deref() != Some(id) {
            return;
        }
        match fs::remove_file(self.directory.join("recovery.json")) {
            Ok(()) => self.recovery_id = None,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.recovery_id = None,
            Err(error) => {
                self.error = Some(
                    i18n::message(|m| &m.error_recovery_remove)
                        .cause("error", error)
                        .into(),
                );
            }
        }
    }
}
