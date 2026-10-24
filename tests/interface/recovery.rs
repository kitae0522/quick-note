use crate::fixtures;
use quick_note::storage::Result;
use quick_note::ui::FocusTarget;

#[test]
fn deletion_request_and_cancel_do_not_write_to_the_database() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    let before: Vec<_> = app
        .store
        .notes()?
        .into_iter()
        .map(|note| (note.id, note.body))
        .collect();
    let current = app.note.id.clone();
    app.request_delete(&current)?;
    assert!(app.pending_delete.is_some());
    app.cancel_delete();
    let after: Vec<_> = app
        .store
        .notes()?
        .into_iter()
        .map(|note| (note.id, note.body))
        .collect();
    assert_eq!(before, after);
    assert_eq!(app.note.id, current);
    Ok(())
}

#[test]
fn save_failure_keeps_unsaved_text_and_blocks_switch_and_inactive_delete() -> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    let active = app.note.id.clone();
    let inactive = app
        .tabs
        .iter()
        .find(|tab| tab.id != active)
        .expect("isolated verification fixture should be valid")
        .id
        .clone();
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch(
        "CREATE TRIGGER fail_write BEFORE UPDATE ON notes BEGIN \
            SELECT RAISE(ABORT, 'test failure'); END;",
    )?;
    app.note.body = "unsaved 아이디어 💡".into();
    app.dirty = true;
    assert!(!app.persist());
    assert!(app.select_note(&inactive, FocusTarget::Editor).is_err());
    assert!(app.new_note().is_err());
    app.request_delete(&inactive)?;
    assert!(app.confirm_delete().is_err());
    assert!(app.store.note(&inactive)?.is_some());
    assert_eq!(app.note.id, active);
    assert_eq!(app.note.body, "unsaved 아이디어 💡");
    assert!(app.dirty);
    connection.execute_batch("DROP TRIGGER fail_write;")?;
    assert!(app.persist());
    assert_eq!(
        app.store
            .note(&active)?
            .expect("isolated verification fixture should be valid")
            .body,
        "unsaved 아이디어 💡"
    );
    Ok(())
}

#[test]
fn failed_sql_save_recovers_text_after_the_app_is_destroyed_and_never_resurrects_a_deleted_note()
-> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    app.note.body = "a saved baseline".into();
    assert!(app.persist());
    let active = app.note.id.clone();
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch(
        "CREATE TRIGGER fail_write BEFORE UPDATE ON notes BEGIN \
            SELECT RAISE(ABORT, 'test failure'); END;",
    )?;
    app.note.body = "unsaved text survives 💡".into();
    app.note.selection = quick_note::model::Selection::from_char_range(&app.note.body, 4, 8);
    let expected_selection = app.note.selection.clone();
    app.dirty = true;
    assert!(!app.persist());
    assert!(directory.path().join("recovery.json").exists());
    let recovery_path = directory.path().join("recovery.json");
    let mut recovery: serde_json::Value = serde_json::from_slice(&std::fs::read(&recovery_path)?)?;
    // A clock adjustment must not discard a failed edit; compare the saved revision.
    recovery["written_at"] = 0.into();
    std::fs::write(&recovery_path, serde_json::to_vec(&recovery)?)?;
    drop(app);
    let mut app = quick_note::app::QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(app.note.id, active);
    assert_eq!(app.note.body, "unsaved text survives 💡");
    assert_eq!(app.note.selection, expected_selection);
    assert!(
        app.dirty,
        "the database still refuses writes; recovered text must remain editable"
    );
    connection.execute_batch("DROP TRIGGER fail_write;")?;
    app.request_delete(&active)?;
    app.confirm_delete()?;
    assert!(!directory.path().join("recovery.json").exists());
    drop(app);
    let app = quick_note::app::QuickNoteApp::try_new(directory.path().to_owned())?;
    assert!(app.store.note(&active)?.is_none());
    assert!(!app.note.body.contains("unsaved text survives"));
    Ok(())
}

#[test]
fn only_one_app_may_edit_the_same_directory() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    assert!(quick_note::app::QuickNoteApp::try_new(directory.path().to_owned()).is_err());
    drop(app);
    assert!(quick_note::app::QuickNoteApp::try_new(directory.path().to_owned()).is_ok());
    Ok(())
}
