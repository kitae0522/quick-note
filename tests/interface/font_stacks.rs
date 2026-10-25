use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use eframe::egui::Role;
use egui_kittest::kittest::Queryable;
use quick_note::appearance;
use quick_note::storage::Result;

#[test]
fn moving_a_font_to_either_end_focuses_its_next_available_action() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_editor_fonts(
        Some(&[
            "Pretendard".into(),
            "Noto Sans CJK".into(),
            "monospace".into(),
        ]),
        &Context::default(),
    )?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose fonts…").click();
    harness.run();
    harness.get_by_label("Move Noto Sans CJK up").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.get_by_label("Move Noto Sans CJK down").is_focused());
    for _ in 0..2 {
        harness.key_press(Key::Enter);
        harness.run();
    }
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard", "monospace", "Noto Sans CJK"]
    );
    assert!(harness.get_by_label("Move Noto Sans CJK up").is_focused());
    Ok(())
}

#[test]
fn reordered_font_focus_remains_inside_a_minimum_window() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_editor_fonts(
        Some(&["monospace".into(), "Pretendard".into()]),
        &Context::default(),
    )?;
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose fonts…").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    harness.get_by_label("Font order · 2/8").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    harness.get_by_label("Move Pretendard up").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    let action = harness.get_by_label("Move Pretendard down");
    assert!(action.is_focused());
    let rect = action.rect();
    let search = harness.get_by_label("Search fonts").rect();
    assert!(
        rect.top() >= search.bottom() + 4.0 && rect.bottom() <= 240.0,
        "{rect:?}, search {search:?}"
    );
    let candidate = harness
        .get_by_role_and_label(Role::CheckBox, "SF Pro")
        .rect();
    assert!(
        harness.ctx.content_rect().contains_rect(candidate),
        "candidate {candidate:?}"
    );
    fixtures::capture(&mut harness, "font-order-focus-clip");
    harness.event(Event::PointerMoved(rect.center()));
    harness.run();
    for pressed in [true, false] {
        harness.event(Event::PointerButton {
            pos: rect.center(),
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        harness.run();
    }
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["monospace", "Pretendard"]
    );
    Ok(())
}

#[test]
fn fonts_can_be_added_reordered_and_removed_without_changing_the_note() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let body = app.note.body.clone();
    let selection = app.note.selection.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose fonts…").click();
    harness.run();
    harness.event(Event::Text("Pretendard".into()));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["monospace", "Pretendard"]
    );
    // Search stays focused after adding; every stack action is reachable by Tab.
    let primary = appearance::font_display_name("monospace");
    for label in [
        "Use theme font".to_owned(),
        format!("Move {primary} down"),
        format!("Remove {primary}"),
        "Move Pretendard up".to_owned(),
    ] {
        harness.key_press(Key::Tab);
        harness.run();
        assert!(harness.get_by_label(&label).is_focused(), "{label}");
    }
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard", "monospace"]
    );
    harness.get_by_label("Search fonts").focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.event(Event::Text("Noto Sans CJK".into()));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard", "monospace", "Noto Sans CJK"]
    );
    let column = harness.get_by_label("Move Pretendard up").rect().left();
    for font in [primary, "Pretendard", "Noto Sans CJK"] {
        let rect = harness.get_by_label(&format!("Move {font} up")).rect();
        assert!((rect.left() - column).abs() <= 1.0, "{font}: {rect:?}");
        assert!(rect.right() <= 780.0);
    }
    fixtures::capture(&mut harness, "ordered-font-stack");
    harness.get_by_label("Move Noto Sans CJK up").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard", "Noto Sans CJK", "monospace"]
    );
    assert!(harness.get_by_label("Move Noto Sans CJK up").is_focused());
    harness.get_by_label(&format!("Remove {primary}")).focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard", "Noto Sans CJK"]
    );
    // The checked search result removes a font too, while the last font stays selected.
    harness
        .get_by(|node| {
            node.role() == Role::CheckBox && node.label().as_deref() == Some("Noto Sans CJK")
        })
        .focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard"]
    );
    assert!(
        harness
            .get_by(|node| {
                node.role() == Role::CheckBox && node.label().as_deref() == Some("Noto Sans CJK")
            })
            .is_focused()
    );
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard", "Noto Sans CJK"]
    );
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard"]
    );
    harness.get_by_label("Remove Pretendard").click();
    harness.run();
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["Pretendard"]
    );
    assert!(harness.state().store.setting("editorFont")?.is_some());
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    assert_eq!(harness.state().note.body, body);
    assert_eq!(harness.state().note.selection, selection);
    Ok(())
}
