//! Storage lifecycle regressions use isolated databases, including rejected transactions.
use quick_note::model::Selection;
use quick_note::storage::Result;
use quick_note::storage::Store;
use rusqlite::Connection;

#[test]
fn version_two_migration_preserves_notes_cursors_order_and_a_private_backup() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.sqlite3");
    let connection = Connection::open(&path)?;
    connection.execute_batch(
        r#"CREATE TABLE notes(id TEXT PRIMARY KEY NOT NULL, body TEXT NOT NULL,
         created_at REAL NOT NULL, updated_at REAL NOT NULL, selection_location INTEGER NOT NULL,
         selection_length INTEGER NOT NULL, title TEXT NOT NULL);
         CREATE TABLE app_state(key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);
         INSERT INTO notes VALUES('one','한💡 original',1,2,3,4,'한💡 original');
         INSERT INTO notes VALUES('two','second',2,3,0,0,'second');
         INSERT INTO app_state VALUES('activeNote','one');
         INSERT INTO app_state VALUES('tabOrder','["two","one"]');
         PRAGMA user_version=2;"#,
    )?;
    drop(connection);
    let mut store = Store::try_new(&path)?;
    let note = store.resume()?;
    assert_eq!(note.id, "one");
    assert_eq!(note.body, "한💡 original");
    assert_eq!(
        note.selection,
        Selection {
            location: 3,
            length: 4
        }
    );
    assert_eq!(
        store
            .summaries()?
            .iter()
            .map(|note| note.id.as_str())
            .collect::<Vec<_>>(),
        ["two", "one"]
    );
    let backup = path.with_extension("v2-backup.sqlite3");
    let old = Connection::open(&backup)?;
    assert_eq!(
        old.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?,
        2
    );
    assert_eq!(
        old.query_row("SELECT body FROM notes WHERE id='one'", [], |row| row
            .get::<_, String>(0))?,
        note.body
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(backup)?.permissions().mode() & 0o777,
            0o600
        );
    }
    Ok(())
}

#[test]
fn close_delete_restore_and_restart_retain_the_same_utf16_document() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.sqlite3");
    let mut store = Store::try_new(&path)?;
    let mut note = store.resume()?;
    note.body = "한💡 café e\u{301}".into();
    note.selection = Selection::from_char_range(&note.body, 1, 5);
    store.save(&note)?;
    let next = store.close_tab(&note.id)?;
    assert_ne!(next.id, note.id);
    assert_eq!(store.all_summaries()?.len(), 2);
    assert_eq!(store.summaries()?.len(), 1);
    assert_eq!(store.activate(&note.id)?.selection, note.selection);
    store.delete_confirmed(&note.id)?;
    assert!(store.note(&note.id)?.is_none());
    assert_eq!(store.deleted_summaries()?.len(), 1);
    assert!(store.save(&note).is_err());
    let restored = store.restore_deleted(&note.id)?;
    assert_eq!(restored.body, note.body);
    assert_eq!(restored.selection, note.selection);
    assert!(store.deleted_summaries()?.is_empty());
    drop(store);
    let mut store = Store::try_new(&path)?;
    assert_eq!(store.resume()?.id, note.id);
    assert_eq!(store.resume()?.selection, note.selection);
    Ok(())
}

#[test]
fn failed_close_and_restore_leave_the_whole_transaction_unchanged() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.sqlite3");
    let mut store = Store::try_new(&path)?;
    let original = store.resume()?;
    let connection = Connection::open(&path)?;
    connection.execute_batch(
        "CREATE TRIGGER reject_active BEFORE INSERT ON app_state WHEN NEW.key='activeNote'
         BEGIN SELECT RAISE(ABORT,'fixture failure'); END;",
    )?;
    assert!(store.close_tab(&original.id).is_err());
    assert_eq!(store.summaries()?.len(), 1);
    assert_eq!(store.all_summaries()?.len(), 1);
    assert_eq!(store.resume()?.id, original.id);
    connection.execute_batch("DROP TRIGGER reject_active;")?;
    store.delete_confirmed(&original.id)?;
    let active = store.resume()?.id;
    connection.execute_batch(
        "CREATE TRIGGER reject_active BEFORE INSERT ON app_state WHEN NEW.key='activeNote'
         BEGIN SELECT RAISE(ABORT,'fixture failure'); END;",
    )?;
    assert!(store.restore_deleted(&original.id).is_err());
    assert_eq!(store.resume()?.id, active);
    assert!(store.note(&original.id)?.is_none());
    assert_eq!(store.deleted_summaries()?.len(), 1);
    Ok(())
}
