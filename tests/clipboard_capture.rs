//! Background text captures preserve the active document and recover failed inserts.
use quick_note::app::QuickNoteApp;
use quick_note::model::Selection;
use quick_note::storage::Result;
use std::fs;

#[test]
fn capture_keeps_active_note_tabs_and_cursor_and_rejects_empty_text() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    app.note.body = "Existing 한💡 text".into();
    app.note.selection = Selection {
        location: 9,
        length: 2,
    };
    assert!(app.persist());
    let current = app.note.clone();
    let tabs = app.tabs.clone();
    let count = app.store.all_summaries()?.len();
    assert!(app.capture_text(String::new()).is_err());
    assert_eq!(app.store.all_summaries()?.len(), count);
    let text = "  # Clipboard 한💡\r\nKeep all whitespace.\n";
    app.capture_text(text.into())?;
    assert_eq!(app.note.id, current.id);
    assert_eq!(app.note.body, current.body);
    assert_eq!(app.note.selection, current.selection);
    assert_eq!(app.tabs, tabs);
    assert_eq!(app.store.summaries()?, tabs);
    assert_eq!(
        app.store.setting("activeNote")?.as_deref(),
        Some(current.id.as_str())
    );
    let recent = app.store.recent_summaries()?;
    let captured = app.store.note(&recent[0].id)?.expect("captured note");
    assert_eq!(captured.body, text);
    assert_eq!(captured.selection.location, text.encode_utf16().count());
    drop(app);
    let restored = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(restored.note.id, current.id);
    assert_eq!(restored.note.selection, current.selection);
    assert_eq!(
        restored
            .store
            .note(&captured.id)?
            .expect("restart capture")
            .body,
        text
    );
    Ok(())
}

#[test]
fn failed_clipboard_insert_recovers_once_and_stale_copy_cannot_resurrect_deletion() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch("CREATE TRIGGER fail_capture BEFORE INSERT ON notes BEGIN SELECT RAISE(ABORT,'clipboard fixture'); END;")?;
    assert!(app.capture_text("Recover clipboard 한💡".into()).is_err());
    assert!(app.dirty);
    let id = app.note.id.clone();
    let recovery = fs::read(directory.path().join("recovery.json"))?;
    assert!(!app.store.contains_note(&id)?);
    drop(app);
    connection.execute_batch("DROP TRIGGER fail_capture")?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(app.note.id, id);
    assert_eq!(app.note.body, "Recover clipboard 한💡");
    assert!(!app.dirty);
    app.request_delete(&id)?;
    app.confirm_delete()?;
    drop(app);
    fs::write(directory.path().join("recovery.json"), recovery)?;
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert!(app.store.note(&id)?.is_none());
    assert!(!directory.path().join("recovery.json").exists());
    Ok(())
}

#[test]
fn activation_failure_rolls_back_the_complete_capture() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch("CREATE TRIGGER fail_active BEFORE INSERT ON app_state WHEN NEW.key='activeNote' BEGIN SELECT RAISE(ABORT,'activation fixture'); END;")?;
    let mut note = quick_note::storage::draft_note();
    note.body = "Atomic capture".into();
    assert!(app.store.capture_note(&note, true).is_err());
    assert!(!app.store.contains_note(&note.id)?);
    assert_eq!(
        app.store.setting("activeNote")?.as_deref(),
        Some(app.note.id.as_str())
    );
    Ok(())
}
