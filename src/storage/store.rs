//! Transactional note operations and metadata queries.
use super::Result;
use super::note_row;
use super::schema;
use super::tab_order;
use crate::i18n;
use crate::model::Note;
use crate::model::NoteSummary;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite::params;
use std::path::Path;

/// SQLite note operations keep draft replacement and active-note updates transactional.
pub struct Store {
    pub(super) connection: Connection,
}

impl Store {
    /// Opens a UTF-8 store, preserving a backup before migrating schema version one.
    pub fn try_new(path: &Path) -> Result<Self> {
        Ok(Self {
            connection: schema::open_connection(path)?,
        })
    }

    pub fn resume(&mut self) -> Result<Note> {
        if let Some(id) = self.setting("activeNote")?
            && let Some(note) = self.note(&id)?
        {
            return Ok(note);
        }
        if let Some(tab) = self.summaries()?.last() {
            let id = tab.id.clone();
            return self.activate(&id);
        }
        self.create_note()
    }

    pub fn note(&self, id: &str) -> Result<Option<Note>> {
        Ok(self
            .connection
            .query_row(
                "SELECT id, body, created_at, updated_at, \
                selection_location, selection_length FROM notes WHERE id = ?1",
                [id],
                note_row::read_note,
            )
            .optional()?)
    }

    pub fn notes(&self) -> Result<Vec<Note>> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, body, \
                created_at, updated_at, selection_location, selection_length FROM \
                notes ORDER BY updated_at DESC, rowid DESC",
        )?;
        Ok(statement
            .query_map([], note_row::read_note)?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn summaries(&self) -> Result<Vec<NoteSummary>> {
        let mut statement = self.connection.prepare_cached(
            "SELECT id, title, updated_at, CASE WHEN title = 'Untitled' THEN body END \
                FROM notes ORDER BY created_at, rowid",
        )?;
        let mut summaries: Vec<NoteSummary> = statement
            .query_map([], |row| {
                Ok(NoteSummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    updated_at: row.get(2)?,
                    is_blank: row
                        .get::<_, Option<String>>(3)?
                        .is_some_and(|body| body.trim().is_empty()),
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let positions = tab_order::positions(&self.connection)?;
        if !positions.is_empty() {
            summaries
                .sort_by_cached_key(|note| positions.get(&note.id).copied().unwrap_or(usize::MAX));
        }
        Ok(summaries)
    }

    pub fn create_note(&mut self) -> Result<Note> {
        let transaction = self.connection.transaction()?;
        let note = note_row::insert_note(&transaction)?;
        transaction.commit()?;
        Ok(note)
    }

    pub fn activate(&mut self, id: &str) -> Result<Note> {
        let note = self
            .note(id)?
            .ok_or(i18n::message(|m| &m.error_note_missing))?;
        self.set_setting("activeNote", id)?;
        Ok(note)
    }

    pub fn save(&self, note: &Note) -> Result<f64> {
        let selection = note.selection.clamped(&note.body);
        let updated_at = super::timestamp();
        let changed = self
            .connection
            .prepare_cached(
                "UPDATE notes SET body = ?1, \
                title = ?2, updated_at = ?3, selection_location = ?4, \
                selection_length = ?5 WHERE id = ?6",
            )?
            .execute(params![
                note.body,
                note.title(),
                updated_at,
                selection.location as i64,
                selection.length as i64,
                note.id
            ])?;
        if changed != 1 {
            return Err(i18n::message(|m| &m.error_note_missing_save).into());
        }
        Ok(updated_at)
    }

    pub fn save_selection(&self, note: &Note) -> Result<()> {
        let selection = note.selection.clamped(&note.body);
        let changed = self
            .connection
            .prepare_cached(
                "UPDATE notes SET selection_location = ?1, selection_length = ?2 WHERE id = ?3",
            )?
            .execute(params![
                selection.location as i64,
                selection.length as i64,
                note.id
            ])?;
        if changed != 1 {
            return Err(i18n::message(|m| &m.error_note_missing_save).into());
        }
        Ok(())
    }

    /// Applies an approved deletion; deleting the final note creates a blank draft atomically.
    pub fn delete_confirmed(&mut self, id: &str) -> Result<Note> {
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
            .ok_or(i18n::message(|m| &m.error_note_missing_delete))?;
        transaction.execute("DELETE FROM notes WHERE id = ?1", [id])?;
        let remaining: Vec<_> = ids.into_iter().filter(|value| value != id).collect();
        let note = if remaining.is_empty() {
            note_row::insert_note(&transaction)?
        } else {
            let next = old_active
                .filter(|active| active != id && remaining.contains(active))
                .unwrap_or_else(|| remaining[index.min(remaining.len() - 1)].clone());
            transaction.execute(
                "INSERT INTO app_state (key, value) VALUES \
                    ('activeNote', ?1) ON CONFLICT(key) DO UPDATE SET \
                    value=excluded.value",
                [&next],
            )?;
            transaction.query_row(
                "SELECT id, body, created_at, updated_at, \
                    selection_location, selection_length FROM notes WHERE id = ?1",
                [next],
                note_row::read_note,
            )?
        };
        transaction.commit()?;
        Ok(note)
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .connection
            .query_row("SELECT value FROM app_state WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.connection.execute(
            "INSERT INTO app_state (key, value) VALUES (?1, ?2) ON \
                CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}
