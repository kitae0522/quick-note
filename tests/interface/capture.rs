//! Quick capture uses the real editor, including save failures and IME transitions.
use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use eframe::egui::ViewportCommand;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;

#[test]
fn compact_capture_keeps_text_cursor_and_controls_then_expands() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let original = app.note.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::N);
    harness.run();
    assert!(harness.state().capture_open());
    assert_eq!(harness.state().note.id, original.id);
    assert_eq!(harness.state().note.selection, original.selection);
    assert!(harness.query_by_label("UTF-8").is_none());
    assert!(harness.get_by_label("Note content").is_focused());
    harness.set_size([320.0, 240.0].into());
    harness.run();
    for label in ["Find a note", "New note", "Note actions"] {
        assert!(harness.get_by_label(label).rect().right() <= 320.0);
    }
    harness.event(Event::Text(" capture 한💡".into()));
    harness.run();
    assert!(!harness.state().dirty);
    let body = harness.state().note.body.clone();
    fixtures::capture(&mut harness, "quick-capture-narrow");
    harness.get_by_label("Note actions").click();
    harness.run();
    harness.get_by_label("Expand note").click();
    harness.run();
    assert!(!harness.state().capture_open());
    assert_eq!(harness.state().note.body, body);
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn escape_saves_the_same_frame_text_before_hiding_and_never_hides_a_failed_save() -> Result<()> {
    for failing in [false, true] {
        let (directory, app) = fixtures::fixture()?;
        let mut harness = fixtures::harness(app, [520.0, 340.0]);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::N);
        harness.run();
        if failing {
            rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?.execute_batch(
                "CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT,'fixture failure'); END;",
            )?;
        }
        harness.input_mut().events.extend([
            Event::Text("last 한글".into()),
            Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            },
        ]);
        harness.step();
        let hidden = harness
            .output()
            .viewport_output
            .values()
            .any(|output| output.commands.contains(&ViewportCommand::Visible(false)));
        assert_eq!(hidden, !failing);
        assert_eq!(harness.state().dirty, failing);
        assert!(harness.state().note.body.contains("last 한글"));
        if !failing {
            assert_eq!(
                harness
                    .state()
                    .store
                    .note(&harness.state().note.id)?
                    .expect("saved note")
                    .body,
                harness.state().note.body
            );
        }
    }
    Ok(())
}

#[test]
fn closing_during_a_hangul_commit_saves_the_complete_syllable() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let id = app.note.id.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "한".into(),
        active_range_chars: None,
    }));
    harness.run();
    harness.input_mut().events.extend([
        Event::Ime(ImeEvent::Commit("한".into())),
        Event::Key {
            key: Key::W,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        },
    ]);
    harness.run();
    assert_ne!(harness.state().note.id, id);
    let body = harness.state().store.note(&id)?.expect("closed note").body;
    assert!(body.contains('한'));
    assert_eq!(body.matches('한').count(), 1);
    Ok(())
}

#[test]
fn deleted_note_restores_through_the_search_ui() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    let original = app.note.clone();
    app.request_delete(&original.id)?;
    app.confirm_delete()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.get_by_label("Recently deleted").click();
    harness.run();
    harness
        .get_by_label("Restore A thought worth keeping")
        .click();
    harness.run();
    assert_eq!(harness.state().note.id, original.id);
    assert_eq!(harness.state().note.body, original.body);
    assert!(harness.state().store.deleted_summaries()?.is_empty());
    Ok(())
}

#[test]
fn focus_mode_keeps_header_actions_keyboard_reachable() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let body = app.note.body.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::F);
    harness.run();
    assert!(harness.query_by_role(Role::Tab).is_none());
    assert!(harness.query_by_label("UTF-8").is_none());
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press(Key::F6);
    harness.run();
    assert!(harness.get_by_label("Find a note").is_focused());
    harness.key_press(Key::F6);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::F);
    harness.run();
    assert_eq!(harness.get_all_by_role(Role::Tab).count(), 3);
    assert_eq!(harness.state().note.body, body);
    Ok(())
}
