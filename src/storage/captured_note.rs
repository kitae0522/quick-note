//! Captures commit the complete UTF-8 note and optional activation in one transaction.
use super::Result;
use super::Store;
use crate::model::Note;
use crate::model::Selection;
use rusqlite::params;

/// A draft has an identity and cursor, but does not create a database row.
#[must_use]
pub fn draft_note() -> Note {
    let time = super::timestamp();
    Note {
        id: uuid::Uuid::new_v4().to_string(),
        body: String::new(),
        created_at: time,
        updated_at: time,
        selection: Selection::default(),
    }
}

impl Store {
    /// Inserts a complete capture. Background captures retain the current note and tabs.
    pub fn capture_note(&mut self, note: &Note, activate: bool) -> Result<f64> {
        let updated_at = super::timestamp();
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO notes(id,body,created_at,updated_at,selection_location,selection_length,title,closed)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![note.id, note.body, note.created_at, updated_at,
                note.selection.location as i64, note.selection.length as i64,
                note.title(), !activate],
        )?;
        if activate {
            transaction.execute(
                "INSERT INTO app_state(key,value) VALUES('activeNote',?1)
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                [&note.id],
            )?;
        }
        transaction.commit()?;
        Ok(updated_at)
    }

    /// Includes deleted notes so a stale recovery file cannot resurrect a committed capture.
    pub fn contains_note(&self, id: &str) -> Result<bool> {
        Ok(self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM notes WHERE id=?1)",
            [id],
            |row| row.get(0),
        )?)
    }
}
