use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::model::Selection;
use quick_note::storage::Result;
use rusqlite::Connection;

#[test]
fn opening_find_waits_until_the_pending_korean_commit_reaches_the_note() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body.clear();
    app.note.selection = Selection::default();
    app.dirty = true;
    assert!(app.persist());
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "한".into(),
        active_range_chars: None,
    }));
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    assert!(harness.query_by_label("Find in note").is_none());
    harness.event(Event::Ime(ImeEvent::Commit("한글".into())));
    harness.run();
    assert_eq!(harness.state().note.body, "한글");
    assert!(harness.get_by_label("Find in note").is_focused());
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "한".into(),
        active_range_chars: None,
    }));
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().note.body, "한글");
    Ok(())
}

#[test]
fn failed_replacement_writes_recover_the_complete_unicode_result() -> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    app.note.body = "한글 💡 한글".into();
    app.note.selection = Selection::default();
    app.dirty = true;
    assert!(app.persist());
    let note_id = app.note.id.clone();
    let database = Connection::open(directory.path().join("notes.sqlite3"))?;
    database.execute_batch("CREATE TRIGGER reject_replacement BEFORE UPDATE OF body ON notes BEGIN SELECT RAISE(ABORT,'fixture'); END;")?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
    harness.run();
    harness.event(Event::Text("한글".into()));
    harness.run();
    harness.get_by_label("Replace with").focus();
    harness.run();
    harness.event(Event::Text("새💡".into()));
    harness.run();
    harness.get_by_label("Replace all").click();
    harness.run();
    assert_eq!(harness.state().note.body, "새💡 💡 새💡");
    assert!(harness.state().dirty);
    assert_eq!(
        harness
            .state()
            .store
            .note(&note_id)?
            .expect("saved fixture")
            .body,
        "한글 💡 한글"
    );
    assert!(directory.path().join("recovery.json").is_file());
    drop(harness);
    database.execute_batch("DROP TRIGGER reject_replacement;")?;
    drop(database);
    let restored = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(restored.note.body, "새💡 💡 새💡");
    assert!(!restored.dirty);
    Ok(())
}

#[test]
fn successive_replacements_undo_the_text_without_cursor_only_steps() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body = "alpha alpha alpha".into();
    app.note.selection = Selection::default();
    app.dirty = true;
    assert!(app.persist());
    let mut harness = fixtures::harness_with_step(app, [920.0, 600.0], 0.05);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
    harness.run();
    harness.event(Event::Text("alpha".into()));
    harness.run();
    harness.get_by_label("Replace with").focus();
    harness.run();
    harness.event(Event::Text("beta".into()));
    harness.run();
    harness.get_by_label("Replace").click();
    harness.run();
    assert_eq!(harness.state().note.body, "beta alpha alpha");
    harness.get_by_label("Replace all").click();
    harness.run();
    assert_eq!(harness.state().note.body, "beta beta beta");
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, "beta alpha alpha");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, "alpha alpha alpha");
    Ok(())
}
