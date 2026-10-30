use crate::fixtures;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;

#[test]
fn keyboard_defaults_cancel_deletion_and_shifted_shortcuts_stay_distinct() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = harness.state().note.id.clone();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Backspace);
    harness.run();
    assert!(harness.get_by_label("Cancel").is_focused());
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.state().pending_delete.is_none());
    assert_eq!(harness.state().store.summaries()?.len(), 3);
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Backspace);
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.state().pending_delete.is_none());
    assert_eq!(harness.state().note.id, original);

    harness.key_press_modifiers(Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert_eq!(harness.state().note.title(), "Weekend ideas");
    harness.key_press_modifiers(Modifiers::CTRL, Key::Tab);
    harness.run();
    assert_eq!(harness.state().note.id, original);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::W);
    harness.run();
    assert!(
        harness.state().pending_delete.is_none(),
        "Close window must not request deletion"
    );
    assert_eq!(harness.state().store.summaries()?.len(), 3);
    Ok(())
}

#[test]
fn roving_note_tabs_preserve_keyboard_focus_and_reveal_the_selected_note() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.get_by_label("A thought worth keeping").focus();
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().note.title(), "Release checklist");
    assert!(harness.get_by_label("Release checklist").is_focused());
    harness.key_press(Key::End);
    harness.run();
    assert_eq!(harness.state().note.title(), "Weekend ideas");
    harness.key_press(Key::Home);
    harness.run();
    assert_eq!(harness.state().note.title(), "A thought worth keeping");
    Ok(())
}
