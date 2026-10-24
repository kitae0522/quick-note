//! Menu actions keep the editor, configured shortcuts and persistence safeguards intact.
use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::app::Mode;
use quick_note::app::QuickNoteApp;
use quick_note::menu::EditCommand;
use quick_note::menu::MenuAction;
use quick_note::shortcut::Command;
use quick_note::storage::Result;

#[test]
fn menu_navigation_waits_for_the_same_frame_hangul_commit() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.new_note()?;
    let original = app.note.id.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "한".into(),
        active_range_chars: None,
    }));
    harness.run();
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    let context = harness.ctx.clone();
    harness
        .state_mut()
        .queue_menu_action(MenuAction::App(Command::New), &context);
    harness.run();
    assert_ne!(harness.state().note.id, original);
    assert!(harness.state().note.body.is_empty());
    assert_eq!(
        harness
            .state()
            .store
            .note(&original)?
            .expect("previous draft")
            .body,
        "한"
    );
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn edit_menu_undo_is_independent_of_an_app_shortcut_bound_to_command_z() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    drop(app);
    std::fs::write(
        directory.path().join("shortcut/bindings.json"),
        r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Z"]}}"#,
    )?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    app.new_note()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Text("a thought".into()));
    harness.run();
    let count = harness.state().tabs.len();
    let context = harness.ctx.clone();
    harness
        .state_mut()
        .queue_menu_action(MenuAction::Edit(EditCommand::Undo), &context);
    harness.run();
    assert!(harness.state().note.body.is_empty());
    assert_eq!(harness.state().tabs.len(), count);
    harness
        .state_mut()
        .queue_menu_action(MenuAction::Edit(EditCommand::Redo), &context);
    harness.run();
    assert_eq!(harness.state().note.body, "a thought");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().tabs.len(), count + 1);
    Ok(())
}

#[test]
fn menus_respect_disabled_preview_and_confirmation_and_settings_delete_policy() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_markdown_enabled(false)?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = harness.state().note.id.clone();
    let count = harness.state().tabs.len();
    let context = harness.ctx.clone();
    harness
        .state_mut()
        .queue_menu_action(MenuAction::App(Command::Preview), &context);
    harness.run();
    assert_eq!(harness.state().mode, Mode::Edit);
    harness
        .state_mut()
        .queue_menu_action(MenuAction::App(Command::Options), &context);
    harness.run();
    assert!(harness.state().settings_open());
    harness
        .state_mut()
        .queue_menu_action(MenuAction::App(Command::Delete), &context);
    harness.run();
    assert!(!harness.state().settings_open());
    assert_eq!(harness.state().note.id, original);
    harness
        .state_mut()
        .queue_menu_action(MenuAction::App(Command::Delete), &context);
    harness.run();
    assert!(harness.state().pending_delete.is_some());
    harness
        .state_mut()
        .queue_menu_action(MenuAction::App(Command::New), &context);
    harness.run();
    assert_eq!(harness.state().tabs.len(), count);
    assert!(harness.get_by_label("Cancel").is_focused());
    Ok(())
}
