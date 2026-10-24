use crate::fixtures;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::app::Mode;
use quick_note::storage::Result;

#[test]
fn edit_search_preview_and_confirmed_delete_follow_real_ui() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    assert_eq!(harness.state().tabs.len(), 3);
    fixtures::capture(&mut harness, "helium-edit");

    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    assert_eq!(harness.state().mode, Mode::Preview);
    assert!(harness.query_by_label("Try next").is_some());
    fixtures::capture(&mut harness, "helium-preview");
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.get_by_label("Search notes").type_text("CAFÉ");
    harness.run();
    assert_eq!(harness.state().search_results().len(), 1);
    assert_eq!(harness.state().search_results()[0].title, "Weekend ideas");
    fixtures::capture(&mut harness, "search");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().note.title(), "Weekend ideas");
    let selected = harness.state().note.id.clone();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert_eq!(
        harness.state().pending_delete.as_deref(),
        Some(selected.as_str())
    );
    assert_eq!(harness.state().store.summaries()?.len(), 3);
    fixtures::capture(&mut harness, "delete-confirmation");
    harness.get_by_label("Cancel").click();
    harness.run();
    assert!(harness.state().pending_delete.is_none());
    assert_eq!(harness.state().note.id, selected);
    assert_eq!(harness.state().store.summaries()?.len(), 3);

    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    harness.get_by_label("Delete note").click();
    harness.run();
    assert!(harness.state().store.note(&selected)?.is_none());
    assert_eq!(harness.state().tabs.len(), 2);
    harness.get_by_label("New note").click();
    harness.run();
    harness
        .get_by_label("Note content")
        .type_text("# UTF-8 proof\n\n가나다 💡 café e\u{301}");
    harness.run();
    let note = &harness.state().note;
    assert!(note.body.contains("가나다 💡 café e\u{301}"));
    assert_eq!(
        harness
            .state()
            .store
            .note(&note.id)?
            .expect("isolated verification fixture should be valid")
            .body,
        note.body
    );
    assert!(!harness.state().dirty);
    Ok(())
}

#[test]
fn narrow_and_dark_modes_keep_controls_reachable() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.get_by_label("New note");
    harness.get_by_label("Find a note");
    harness.get_by_label("Open settings");
    for label in ["New note", "Find a note", "Open settings"] {
        let rect = harness.get_by_label(label).rect();
        assert!(
            rect.left() >= 0.0 && rect.right() <= 320.0,
            "{label} must stay inside the narrow window: {rect:?}"
        );
        assert!(
            rect.width() >= 24.0 && rect.height() >= 24.0,
            "{label} target size"
        );
    }
    fixtures::capture(&mut harness, "narrow-edit");
    let context = harness.ctx.clone();
    harness.state_mut().set_theme("graphite", &context)?;
    harness.run();
    fixtures::capture(&mut harness, "graphite-narrow");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness
        .get_by_label("Search notes")
        .type_text("no matching note");
    harness.run();
    let clear = harness.get_by_label("Clear search").rect();
    assert!(
        clear.right() <= 320.0 && clear.bottom() <= 240.0,
        "search actions must stay inside the minimum window: {clear:?}"
    );
    harness.get_by_label("Clear search").click();
    harness.run();
    assert!(!harness.state().search_results().is_empty());
    fixtures::capture(&mut harness, "narrow-search");
    Ok(())
}
