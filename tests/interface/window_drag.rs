use crate::fixtures;
use eframe::egui;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use eframe::egui::ViewportCommand;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;

#[test]
fn empty_window_drag_space_is_not_a_keyboard_stop() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.tabs.truncate(1);
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.get_by_label("New note").focus();
    harness.run();
    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label("Open settings").is_focused());
    harness.get_by_label("Search notes").focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert!(
        harness
            .query_by(
                |node| node.is_focused() && node.label().is_some_and(|label| !label.is_empty())
            )
            .is_some()
    );
    Ok(())
}

#[test]
fn window_drag_hands_pointer_to_the_os_once_and_does_not_block_typing() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.tabs.truncate(1);
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let position = egui::pos2(800.0, 22.0);
    harness.hover_at(position);
    harness.step();
    harness.event(Event::PointerButton {
        pos: position,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    harness.step();
    assert!(
        harness
            .output()
            .viewport_output
            .values()
            .any(|output| output.commands.contains(&ViewportCommand::StartDrag))
    );
    assert!(!harness.ctx.input(|input| input.pointer.primary_down()));
    assert!(harness.ctx.dragged_id().is_none());
    harness.hover_at(position + egui::vec2(40.0, 0.0));
    harness.step();
    assert!(
        !harness
            .output()
            .viewport_output
            .values()
            .any(|output| output.commands.contains(&ViewportCommand::StartDrag))
    );
    harness.event(Event::Text(" still editable".into()));
    harness.run();
    assert!(harness.state().note.body.contains(" still editable"));
    assert!(!harness.state().dirty);
    Ok(())
}

#[test]
fn moving_into_the_header_does_not_convert_a_canvas_press_into_window_drag() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let from = egui::pos2(5.0, 500.0);
    harness.hover_at(from);
    harness.step();
    harness.drag_at(from);
    harness.step();
    harness.hover_at(egui::pos2(850.0, 2.0));
    harness.step();
    assert!(
        !harness
            .output()
            .viewport_output
            .values()
            .any(|output| output.commands.contains(&ViewportCommand::StartDrag))
    );
    Ok(())
}

#[test]
fn tabs_search_and_header_buttons_never_start_native_window_drag() -> Result<()> {
    for label in [
        "Weekend ideas",
        "A thought worth keeping",
        "Search notes",
        "New note",
        "Open settings",
        "Close Weekend ideas",
    ] {
        let (_directory, app) = fixtures::fixture()?;
        let mut harness = fixtures::harness(app, [920.0, 600.0]);
        harness.run();
        let from = harness.get_by_label(label).rect().center();
        harness.hover_at(from);
        harness.step();
        harness.drag_at(from);
        harness.step();
        assert!(
            !harness
                .output()
                .viewport_output
                .values()
                .any(|output| output.commands.contains(&ViewportCommand::StartDrag)),
            "{label}"
        );
        harness.hover_at(from + egui::vec2(16.0, 0.0));
        harness.step();
        assert!(
            !harness
                .output()
                .viewport_output
                .values()
                .any(|output| output.commands.contains(&ViewportCommand::StartDrag)),
            "{label}"
        );
    }
    Ok(())
}
