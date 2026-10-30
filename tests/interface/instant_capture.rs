//! Fresh captures only become notes on input, and never lose a failed first write.
use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;

#[test]
fn unused_capture_restores_previous_note_without_any_database_change() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let original = app.note.clone();
    let before = app.store.all_summaries()?;
    let mut harness = fixtures::harness(app, [520.0, 340.0]);
    harness.run();
    for _ in 0..3 {
        harness.key_press_modifiers(
            Modifiers::COMMAND | Modifiers::ALT | Modifiers::SHIFT,
            Key::N,
        );
        harness.run();
        assert!(harness.state().capture_open());
        assert!(harness.state().note.body.is_empty());
        assert!(harness.get_by_label("Note content").is_focused());
        assert!(
            !harness
                .state()
                .store
                .contains_note(&harness.state().note.id)?
        );
        assert_eq!(
            harness.state().store.setting("activeNote")?.as_deref(),
            Some(original.id.as_str())
        );
        harness.key_press(Key::Escape);
        harness.run();
        assert_eq!(harness.state().note.id, original.id);
        assert_eq!(harness.state().note.selection, original.selection);
        assert_eq!(harness.state().store.all_summaries()?, before);
    }
    Ok(())
}

#[test]
fn first_text_creates_exactly_one_durable_note_with_utf16_cursor() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let count = app.store.all_summaries()?.len();
    let mut harness = fixtures::harness(app, [520.0, 340.0]);
    harness.run();
    harness.key_press_modifiers(
        Modifiers::COMMAND | Modifiers::ALT | Modifiers::SHIFT,
        Key::N,
    );
    harness.run();
    let id = harness.state().note.id.clone();
    harness.event(Event::Text("한💡 café".into()));
    harness.run();
    let saved = harness
        .state()
        .store
        .note(&id)?
        .expect("first input is committed");
    assert_eq!(saved.body, "한💡 café");
    assert_eq!(saved.selection.location, saved.body.encode_utf16().count());
    assert_eq!(
        harness.state().store.setting("activeNote")?.as_deref(),
        Some(id.as_str())
    );
    harness.event(Event::Text("!".into()));
    harness.run();
    assert_eq!(harness.state().store.all_summaries()?.len(), count + 1);
    assert_eq!(harness.state().note.id, id);
    Ok(())
}

#[test]
fn failed_first_input_survives_restart_and_emptying_it_discards_recovery() -> Result<()> {
    for empty_before_restart in [false, true] {
        let (directory, app) = fixtures::fixture()?;
        let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
        connection.execute_batch("CREATE TRIGGER fail_capture BEFORE INSERT ON notes BEGIN SELECT RAISE(ABORT,'capture fixture'); END;")?;
        let mut harness = fixtures::harness(app, [520.0, 340.0]);
        harness.run();
        harness.key_press_modifiers(
            Modifiers::COMMAND | Modifiers::ALT | Modifiers::SHIFT,
            Key::N,
        );
        harness.run();
        let id = harness.state().note.id.clone();
        harness.event(Event::Text("Recover 한💡".into()));
        harness.run();
        assert!(harness.state().dirty);
        assert!(!harness.state().store.contains_note(&id)?);
        assert!(directory.path().join("recovery.json").exists());
        if empty_before_restart {
            harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
            harness.key_press(Key::Backspace);
            harness.run();
            assert!(!harness.state().dirty);
            assert!(!directory.path().join("recovery.json").exists());
        }
        drop(harness);
        connection.execute_batch("DROP TRIGGER fail_capture")?;
        let restored = QuickNoteApp::try_new(directory.path().to_owned())?;
        if empty_before_restart {
            assert!(!restored.store.contains_note(&id)?);
        } else {
            assert_eq!(restored.note.id, id);
            assert_eq!(restored.note.body, "Recover 한💡");
            assert!(!restored.dirty);
        }
        assert!(!directory.path().join("recovery.json").exists());
    }
    Ok(())
}

#[test]
fn selecting_a_recent_closed_note_preserves_unsaved_text() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.capture_text("Clipboard note 한글".into())?;
    let id = app.store.recent_summaries()?[0].id.clone();
    let previous = app.note.id.clone();
    app.note.body = "Pending edit".into();
    app.dirty = true;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let context = harness.ctx.clone();
    harness
        .state_mut()
        .queue_menu_action(quick_note::menu::MenuAction::OpenNote(id.clone()), &context);
    harness.run();
    assert_eq!(harness.state().note.id, id);
    assert_eq!(
        harness
            .state()
            .store
            .note(&previous)?
            .expect("previous note")
            .body,
        "Pending edit"
    );
    assert!(harness.state().tabs.iter().any(|tab| tab.id == id));
    Ok(())
}

#[test]
fn background_quit_saves_or_blocks_before_requesting_a_window_close() -> Result<()> {
    for failing in [false, true] {
        let (directory, mut app) = fixtures::fixture()?;
        let context = eframe::egui::Context::default();
        if failing {
            rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?.execute_batch(
                "CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT,'quit fixture'); END;",
            )?;
        }
        app.note.body = "Pending background draft".into();
        app.dirty = true;
        app.queue_menu_action(
            quick_note::menu::MenuAction::App(quick_note::shortcut::Command::Quit),
            &context,
        );
        let output = context.run_logic(&Default::default(), |context| app.background(context));
        let closed = output
            .viewport_commands
            .values()
            .any(|commands| commands.contains(&eframe::egui::ViewportCommand::Close));
        assert_eq!(closed, !failing);
        assert_eq!(app.dirty, failing);
        assert_eq!(directory.path().join("recovery.json").exists(), failing);
        if !failing {
            assert_eq!(
                app.store
                    .note(&app.note.id)?
                    .expect("saved before quit")
                    .body,
                app.note.body
            );
        }
    }
    Ok(())
}
