//! Closing and deleting affect visibility; note text survives both operations.
use super::Result;
use super::Store;
use super::note_row;
use super::tab_order;
use crate::i18n;
use crate::model::Note;
use rusqlite::OptionalExtension;
use rusqlite::params;

impl Store {
    /// Closes a tab atomically, leaving its note searchable and its cursor intact.
    pub fn close_tab(&mut self, id: &str) -> Result<Note> {
        self.dismiss_note(id, false)
    }

    /// Moves a note to Recently deleted, retaining its ID, text and selection.
    pub fn delete_confirmed(&mut self, id: &str) -> Result<Note> {
        self.dismiss_note(id, true)
    }

    fn dismiss_note(&mut self, id: &str, deleted: bool) -> Result<Note> {
        let transaction = self.connection.transaction()?;
        let old_active: Option<String> = transaction
            .query_row(
                "SELECT value FROM app_state WHERE key = 'activeNote'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let ids = tab_order::ordered_ids(&transaction)?;
        let index = ids
            .iter()
            .position(|value| value == id)
            .ok_or(i18n::message(|m| &m.error_note_missing))?;
        transaction.execute(
            "UPDATE notes SET closed = 1, deleted_at = ?1 WHERE id = ?2",
            params![deleted.then(super::timestamp), id],
        )?;
        let remaining: Vec<_> = ids.into_iter().filter(|value| value != id).collect();
        let note = if remaining.is_empty() {
            note_row::insert_note(&transaction)?
        } else {
            let next = old_active
                .filter(|active| active != id && remaining.contains(active))
                .unwrap_or_else(|| remaining[index.min(remaining.len() - 1)].clone());
            transaction.execute(
                "INSERT INTO app_state (key, value) VALUES ('activeNote', ?1) \
                ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                [&next],
            )?;
            transaction.query_row(
                "SELECT id, body, created_at, updated_at, selection_location, selection_length \
                FROM notes WHERE id = ?1",
                [&next],
                note_row::read_note,
            )?
        };
        transaction.commit()?;
        Ok(note)
    }

    /// Restores a deleted note and activates it in the same transaction.
    pub fn restore_deleted(&mut self, id: &str) -> Result<Note> {
        let transaction = self.connection.transaction()?;
        let changed = transaction.execute(
            "UPDATE notes SET deleted_at = NULL, closed = 0 WHERE id = ?1 AND deleted_at IS NOT NULL",
            [id],
        )?;
        if changed != 1 {
            return Err(i18n::message(|m| &m.error_note_missing).into());
        }
        transaction.execute(
            "INSERT INTO app_state (key, value) VALUES ('activeNote', ?1) \
            ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [id],
        )?;
        let note = transaction.query_row(
            "SELECT id, body, created_at, updated_at, selection_location, selection_length \
            FROM notes WHERE id = ?1",
            [id],
            note_row::read_note,
        )?;
        transaction.commit()?;
        Ok(note)
    }
}
