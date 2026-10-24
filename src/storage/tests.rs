use super::Result;
use super::Store;
use crate::model::Selection;
use rusqlite::Connection;
use std::fs;

#[test]
fn edits_commit_utf8_before_close_and_restore_selection() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("notes.sqlite3");
    let mut writer = Store::try_new(&path)?;
    let mut note = writer.resume()?;
    let mut reader = Store::try_new(&path)?;
    for text in ["á", "아이디어 💡", "last\0character\n끝"] {
        note.body = text.into();
        note.selection =
            Selection::from_char_range(text, text.chars().count(), text.chars().count());
        writer.save(&note)?;
        assert_eq!(reader.resume()?.body, text);
        assert_eq!(reader.resume()?.selection, note.selection);
    }
    assert_eq!(
        writer
            .connection
            .pragma_query_value(None, "encoding", |row| row.get::<_, String>(0))?,
        "UTF-8"
    );
    assert_eq!(reader.summaries()?.len(), 1);
    Ok(())
}

#[test]
fn legacy_migration_preserves_data_ids_order_cursor_and_backup() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("notes.sqlite3");
    {
        let connection = Connection::open(&path)?;
        connection.execute_batch(
            "CREATE TABLE notes(id TEXT PRIMARY KEY NOT NULL,body \
                TEXT NOT NULL,created_at REAL NOT NULL,updated_at REAL NOT \
                NULL,selection_location INTEGER NOT NULL,selection_length INTEGER NOT \
                NULL); CREATE TABLE app_state(key TEXT PRIMARY KEY NOT NULL,value \
                TEXT NOT NULL); PRAGMA user_version=1;",
        )?;
        connection.execute(
            "INSERT INTO notes VALUES ('legacy',?1,1,2,3,0)",
            ["# 아이디어 💡\noriginal"],
        )?;
        connection.execute("INSERT INTO app_state VALUES ('activeNote','legacy')", [])?;
    }
    let mut store = Store::try_new(&path)?;
    let note = store.resume()?;
    assert_eq!(note.id, "legacy");
    assert_eq!(note.body, "# 아이디어 💡\noriginal");
    assert_eq!(note.selection.location, 3);
    assert_eq!(store.summaries()?[0].title, "아이디어 💡");
    let backup = Connection::open(path.with_extension("v1-backup.sqlite3"))?;
    assert_eq!(
        backup.query_row("SELECT body FROM notes", [], |row| row.get::<_, String>(0))?,
        note.body
    );
    assert_eq!(
        backup.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?,
        1
    );
    Ok(())
}

#[test]
fn deletion_replaces_last_note_and_does_not_switch_for_an_inactive_deletion() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let mut store = Store::try_new(&temp.path().join("notes.sqlite3"))?;
    let first = store.resume()?;
    let second = store.create_note()?;
    let active = store.delete_confirmed(&first.id)?;
    assert_eq!(active.id, second.id);
    let replacement = store.delete_confirmed(&second.id)?;
    assert_ne!(replacement.id, second.id);
    assert!(replacement.body.is_empty());
    assert_eq!(store.summaries()?.len(), 1);
    assert!(store.note(&first.id)?.is_none());
    Ok(())
}

#[test]
fn write_failure_and_corruption_never_reset_the_store() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("notes.sqlite3");
    let mut store = Store::try_new(&path)?;
    let mut note = store.resume()?;
    note.body = "keep".into();
    store.save(&note)?;
    store.connection.execute_batch(
        "CREATE TRIGGER fail_write BEFORE UPDATE ON notes \
            BEGIN SELECT RAISE(ABORT,'test failure'); END;",
    )?;
    note.body = "unsaved".into();
    assert!(store.save(&note).is_err());
    assert_eq!(store.resume()?.body, "keep");
    assert!(store.activate("missing").is_err());
    let corrupt = temp.path().join("corrupt.sqlite3");
    fs::write(&corrupt, b"not a database")?;
    assert!(Store::try_new(&corrupt).is_err());
    assert_eq!(fs::read(corrupt)?, b"not a database");
    Ok(())
}
