use crate::fixtures;
use crate::tab_input;
use eframe::egui;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use quick_note::storage::Store;

#[test]
fn keyboard_moves_current_tab_and_navigation_follows_the_saved_order() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = tab_input::ids(&harness);
    let body = harness.state().note.body.clone();
    let modifiers = Modifiers::CTRL | Modifiers::SHIFT;
    harness.key_press_modifiers(modifiers, Key::PageDown);
    harness.run();
    assert_eq!(
        tab_input::ids(&harness),
        vec![
            original[1].clone(),
            original[0].clone(),
            original[2].clone()
        ]
    );
    assert_eq!(harness.state().note.id, original[0]);
    assert_eq!(harness.state().note.body, body);
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press_modifiers(modifiers, Key::PageDown);
    harness.run();
    let moved = vec![
        original[1].clone(),
        original[2].clone(),
        original[0].clone(),
    ];
    assert_eq!(tab_input::ids(&harness), moved);
    harness.key_press_modifiers(modifiers, Key::PageDown);
    harness.run();
    assert_eq!(tab_input::ids(&harness), moved);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    harness.run();
    assert_eq!(harness.state().note.id, original[1]);
    harness.key_press_modifiers(Modifiers::CTRL, Key::Tab);
    harness.run();
    assert_eq!(harness.state().note.id, original[2]);
    harness.key_press(Key::F6);
    harness.run();
    harness.key_press_modifiers(modifiers, Key::PageUp);
    harness.run();
    assert_eq!(
        tab_input::ids(&harness),
        vec![
            original[2].clone(),
            original[1].clone(),
            original[0].clone()
        ]
    );
    assert!(harness.get_by_label("Weekend ideas").is_focused());
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().note.id, original[1]);
    assert!(harness.get_by_label("Release checklist").is_focused());
    Ok(())
}

#[test]
fn edge_scroll_keeps_a_drag_alive_when_its_source_tab_is_virtualized_out() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut store = Store::try_new(&directory.path().join("notes.sqlite3"))?;
    let mut first = String::new();
    for index in 0..20 {
        let mut note = store.create_note()?;
        note.body = format!("Idea {index:02}");
        store.save(&note)?;
        if index == 0 {
            first = note.id;
        }
    }
    store.activate(&first)?;
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = tab_input::ids(&harness);
    let rect = harness.get_by_label("Notes").rect();
    let from = harness.get_by_label("Idea 00").rect().center();
    let right = egui::pos2(rect.right() - 2.0, from.y);
    tab_input::start(&mut harness, from, right);
    for _ in 0..480 {
        harness.step();
    }
    assert!(harness.state().header.scroll > 1000.0);
    assert!(harness.query_by_label("Idea 00").is_none());
    assert!(harness.query_by_label("Idea 19").is_some());
    assert!(harness.get_all_by_role(Role::Tab).count() < 10);
    assert!(harness.state().store.setting("tabOrder")?.is_none());
    fixtures::capture(&mut harness, "tab-drag-edge-scroll");
    tab_input::finish(&mut harness, right);
    let mut expected = original[1..].to_vec();
    expected.push(original[0].clone());
    assert_eq!(tab_input::ids(&harness), expected);
    assert_eq!(harness.state().note.id, first);

    let from = harness.get_by_label("Idea 00").rect().center();
    let left = egui::pos2(rect.left() + 2.0, from.y);
    tab_input::start(&mut harness, from, left);
    for _ in 0..480 {
        harness.step();
    }
    assert!(harness.state().header.scroll < 1.0);
    tab_input::finish(&mut harness, left);
    assert_eq!(tab_input::ids(&harness), original);
    assert_eq!(harness.state().note.id, first);
    Ok(())
}
