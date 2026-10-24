//! Tab placement is UI metadata; moving a tab never edits the underlying note.
use super::Result;
use super::Store;
use crate::i18n;
use rusqlite::Connection;
use std::collections::HashMap;

pub(super) fn positions(connection: &Connection) -> Result<HashMap<String, usize>> {
    use rusqlite::OptionalExtension;

    let stored: Option<String> = connection
        .query_row(
            "SELECT value FROM app_state WHERE key = 'tabOrder'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    let ids: Vec<String> = match stored {
        Some(json) => serde_json::from_str(&json)?,
        None => Vec::new(),
    };
    Ok(ids
        .into_iter()
        .enumerate()
        .map(|(index, id)| (id, index))
        .collect())
}

pub(super) fn ordered_ids(connection: &Connection) -> Result<Vec<String>> {
    let mut ids: Vec<String> = connection
        .prepare_cached("SELECT id FROM notes ORDER BY created_at, rowid")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let positions = positions(connection)?;
    if !positions.is_empty() {
        ids.sort_by_cached_key(|id| positions.get(id).copied().unwrap_or(usize::MAX));
    }
    Ok(ids)
}

impl Store {
    /// Commits one final position atomically. Unchanged positions perform no write.
    pub fn move_tab(&self, id: &str, index: usize) -> Result<bool> {
        let mut ids = ordered_ids(&self.connection)?;
        let previous = ids
            .iter()
            .position(|value| value == id)
            .ok_or(i18n::message(|m| &m.error_note_missing))?;
        let index = index.min(ids.len() - 1);
        if previous == index {
            return Ok(false);
        }
        let moved = ids.remove(previous);
        ids.insert(index, moved);
        self.set_setting("tabOrder", &serde_json::to_string(&ids)?)?;
        Ok(true)
    }
}
