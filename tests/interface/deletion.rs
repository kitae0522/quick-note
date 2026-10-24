use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use quick_note::ui::FocusTarget;

#[test]
fn command_w_deletes_empty_and_whitespace_notes_without_confirmation() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original: Vec<_> = harness
        .state()
        .store
        .notes()?
        .into_iter()
        .map(|note| (note.id, note.body))
        .collect();
    for body in ["", " \t\n\u{3000}"] {
        harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
        harness.run();
        if !body.is_empty() {
            harness.event(Event::Paste(body.into()));
            harness.run();
        }
        let id = harness.state().note.id.clone();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
        harness.run();
        assert!(harness.state().pending_delete.is_none());
        assert!(harness.state().store.note(&id)?.is_none());
        assert_eq!(harness.state().tabs.len(), original.len());
        assert!(harness.get_by_label("Note content").is_focused());
        for (id, body) in &original {
            assert_eq!(
                harness
                    .state()
                    .store
                    .note(id)?
                    .expect("original fixture note")
                    .body,
                *body
            );
        }
    }
    Ok(())
}

#[test]
fn deleting_the_final_empty_note_leaves_a_fresh_editable_draft() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let id = app.note.id.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert!(harness.state().pending_delete.is_none());
    assert!(harness.state().store.note(&id)?.is_none());
    assert_ne!(harness.state().note.id, id);
    assert_eq!(harness.state().tabs.len(), 1);
    assert!(harness.state().note.body.is_empty());
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn a_stored_empty_note_with_unsaved_text_still_requires_confirmation() -> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    app.new_note()?;
    let id = app.note.id.clone();
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch(
        "CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN \
            SELECT RAISE(ABORT, 'test failure'); END;",
    )?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Text("unsaved 아이디어".into()));
    harness.run();
    assert!(harness.state().dirty);
    assert!(
        harness
            .state()
            .store
            .note(&id)?
            .expect("stored blank fixture")
            .body
            .is_empty()
    );
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert_eq!(harness.state().pending_delete.as_deref(), Some(id.as_str()));
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().note.id, id);
    assert_eq!(harness.state().note.body, "unsaved 아이디어");
    Ok(())
}

#[test]
fn closing_an_inactive_empty_tab_preserves_the_current_note() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    let active = app.note.id.clone();
    app.new_note()?;
    let blank = app.note.id.clone();
    app.select_note(&active, FocusTarget::Editor)?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.get_by_label("Delete Untitled").click();
    harness.run();
    assert!(harness.state().pending_delete.is_none());
    assert!(harness.state().store.note(&blank)?.is_none());
    assert_eq!(harness.state().note.id, active);
    Ok(())
}
