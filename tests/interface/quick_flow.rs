//! The capture notebook keeps closed notes, restores deletions and finds body context.
use crate::fixtures;

use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use quick_note::ui::FocusTarget;

#[test]
fn closing_a_tab_preserves_text_and_reopens_it_after_restart() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let id = app.note.id.clone();
    let body = app.note.body.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert!(harness.state().pending_delete.is_none());
    assert_eq!(harness.state().tabs.len(), 2);
    assert_eq!(
        harness.state().store.note(&id)?.expect("closed note").body,
        body
    );
    drop(harness);
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(app.tabs.len(), 2);
    app.header.query = "thought worth keeping".into();
    app.refresh_search()?;
    assert_eq!(app.search_results()[0].id, id);
    app.select_note(&id, FocusTarget::Editor)?;
    assert_eq!(app.tabs.len(), 3);
    assert_eq!(app.note.body, body);
    Ok(())
}

#[test]
fn confirmed_deletion_keeps_a_recoverable_copy() -> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    let id = app.note.id.clone();
    let body = app.note.body.clone();
    app.request_delete(&id)?;
    app.confirm_delete()?;
    assert!(app.store.note(&id)?.is_none());
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    let retained: String =
        connection.query_row("SELECT body FROM notes WHERE id=?1", [&id], |row| {
            row.get(0)
        })?;
    assert_eq!(retained, body);
    Ok(())
}

#[test]
fn closing_a_tab_never_discards_a_failed_save() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let id = app.note.id.clone();
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT, 'fixture failure'); END;")?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Text("unsaved 한글 💡".into()));
    harness.run();
    let body = harness.state().note.body.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert!(harness.state().dirty);
    assert!(harness.state().pending_delete.is_none());
    assert_eq!(harness.state().note.id, id);
    assert_eq!(harness.state().note.body, body);
    assert_eq!(harness.state().tabs.len(), 3);
    Ok(())
}

#[test]
fn search_shows_the_matching_sentence_and_opens_that_match() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.event(Event::Text("café".into()));
    harness.run();
    harness.get_by_label_contains("A café with a quiet");
    harness.key_press(Key::Enter);
    harness.run();
    let note = &harness.state().note;
    let (start, end) = note.selection.char_range(&note.body);
    assert_eq!(
        note.body
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>(),
        "café"
    );
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}
