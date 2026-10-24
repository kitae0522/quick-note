use crate::fixtures;
use eframe::egui::Key;
use eframe::egui::KeyboardShortcut;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use std::fs;

#[test]
fn startup_creates_an_editable_shortcut_file_without_overwriting_it() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let path = directory.path().join("shortcut/bindings.json");
    assert!(
        path.is_file(),
        "app shortcuts must be editable outside the binary"
    );
    drop(app);
    let source = r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Shift+N"],"search_notes":["Primary+K"]}}"#;
    fs::write(&path, source)?;
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(fs::read_to_string(&path)?, source);
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = harness.state().note.id.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run();
    assert_eq!(
        harness.state().note.id,
        original,
        "the old binding must be removed"
    );
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::N);
    harness.run();
    assert_ne!(harness.state().note.id, original);
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    assert!(harness.get_by_label("Search notes").is_focused());
    Ok(())
}

#[test]
fn shortcuts_reload_from_settings_and_reject_conflicts_without_losing_work() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = harness.state().note.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
    harness.run();
    harness.get_by_role_and_label(Role::Button, "Open shortcuts folder");
    let path = directory.path().join("shortcut/bindings.json");
    fs::write(
        &path,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Shift+N"]}}"#,
    )?;
    harness
        .get_by_role_and_label(Role::Button, "Reload shortcuts")
        .click();
    harness.run();
    let keys = harness.ctx.format_shortcut(&KeyboardShortcut::new(
        Modifiers::COMMAND | Modifiers::SHIFT,
        Key::N,
    ));
    harness.get_by_role_and_label(Role::Label, &keys);
    fixtures::capture(&mut harness, "shortcut-settings-custom");
    fs::write(
        &path,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+W"]}}"#,
    )?;
    harness
        .get_by_role_and_label(Role::Button, "Reload shortcuts")
        .click();
    harness.run();
    assert_eq!(harness.state().note.body, original.body);
    assert_eq!(harness.state().note.id, original.id);
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::N);
    harness.run();
    assert_ne!(
        harness.state().note.id,
        original.id,
        "last valid bindings remain active"
    );
    assert_eq!(
        harness
            .state()
            .store
            .note(&original.id)?
            .expect("original note")
            .body,
        original.body
    );
    Ok(())
}

#[test]
fn returning_to_the_app_applies_edits_and_disabled_actions_without_restarting() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let original = app.note.clone();
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.input_mut().focused = false;
    harness.run();
    fs::write(
        directory.path().join("shortcut/bindings.json"),
        r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Shift+N"],"toggle_preview":[]}}"#,
    )?;
    harness.input_mut().focused = true;
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run();
    assert_eq!(harness.state().note.id, original.id);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    assert_eq!(harness.state().mode, quick_note::app::Mode::Edit);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::N);
    harness.run();
    assert_ne!(harness.state().note.id, original.id);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
    harness.run();
    let context = harness.ctx.clone();
    harness
        .state_mut()
        .set_language(quick_note::i18n::Language::Korean, &context)?;
    harness.run();
    harness.get_by_role_and_label(Role::Button, "단축키 다시 불러오기");
    fixtures::capture(&mut harness, "shortcut-settings-narrow-ko");
    assert_eq!(
        harness
            .state()
            .store
            .note(&original.id)?
            .expect("original note")
            .body,
        original.body
    );
    Ok(())
}

#[test]
fn an_invalid_startup_configuration_keeps_defaults_and_saved_notes_available() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let original = app.note.clone();
    drop(app);
    fs::write(directory.path().join("shortcut/bindings.json"), "{broken")?;
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.get_by_label_contains("Defaults are active");
    assert_eq!(harness.state().note.body, original.body);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run();
    assert_ne!(harness.state().note.id, original.id);
    assert_eq!(
        fs::read_to_string(directory.path().join("shortcut/bindings.json"))?,
        "{broken"
    );
    Ok(())
}
