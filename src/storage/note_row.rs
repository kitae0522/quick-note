//! Convert SQLite rows and insert a draft within the caller's transaction.
use super::Result;
use crate::model::Note;
use crate::model::Selection;
use rusqlite::Row;
use rusqlite::Transaction;
use rusqlite::params;

pub(super) fn read_note(row: &Row<'_>) -> rusqlite::Result<Note> {
    let body: String = row.get(1)?;
    let selection = Selection {
        location: row.get::<_, i64>(4)?.max(0) as usize,
        length: row.get::<_, i64>(5)?.max(0) as usize,
    }
    .clamped(&body);
    Ok(Note {
        id: row.get(0)?,
        body,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        selection,
    })
}

pub(super) fn insert_note(transaction: &Transaction<'_>) -> Result<Note> {
    let time = super::timestamp();
    let note = Note {
        id: uuid::Uuid::new_v4().to_string(),
        body: String::new(),
        created_at: time,
        updated_at: time,
        selection: Selection::default(),
    };
    transaction.execute(
        "INSERT INTO notes (id, body, created_at, updated_at, title) \
            VALUES (?1, '', ?2, ?2, 'Untitled')",
        params![note.id, time],
    )?;
    transaction.execute(
        "INSERT INTO app_state (key, value) VALUES ('activeNote', ?1) \
            ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [&note.id],
    )?;
    Ok(note)
}
