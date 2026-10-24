use crate::fixtures;
use crate::tab_input;
use eframe::egui;
use eframe::egui::DragAndDrop;
use eframe::egui::Event;
use eframe::egui::Key;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use rusqlite::Connection;

#[test]
fn dragging_active_and_inactive_tabs_commits_on_drop_and_retains_the_draft() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = tab_input::ids(&harness);
    let active = harness.state().note.id.clone();
    let body = harness.state().note.body.clone();
    let selection = harness.state().note.selection.clone();
    let first = harness.get_by_label("A thought worth keeping").rect();
    let last = harness.get_by_label("Weekend ideas").rect();
    let target = egui::pos2(first.left() + 2.0, first.center().y);
    tab_input::start(&mut harness, last.center(), target);
    assert!(DragAndDrop::has_any_payload(&harness.ctx));
    assert_eq!(tab_input::ids(&harness), original);
    assert!(harness.state().store.setting("tabOrder")?.is_none());
    fixtures::capture(&mut harness, "tab-drag-insertion");
    tab_input::finish(&mut harness, target);
    let reordered = vec![
        original[2].clone(),
        original[0].clone(),
        original[1].clone(),
    ];
    assert_eq!(tab_input::ids(&harness), reordered);
    assert_eq!(harness.state().note.id, active);
    assert_eq!(harness.state().note.body, body);
    assert_eq!(harness.state().note.selection, selection);
    assert!(harness.get_by_label("Note content").is_focused());

    let from = harness
        .get_by_label("A thought worth keeping")
        .rect()
        .center();
    let end = harness.get_by_label("Release checklist").rect();
    let target = egui::pos2(end.right() - 2.0, end.center().y);
    tab_input::start(&mut harness, from, target);
    tab_input::finish(&mut harness, target);
    let expected = vec![
        original[2].clone(),
        original[1].clone(),
        original[0].clone(),
    ];
    assert_eq!(tab_input::ids(&harness), expected);
    assert_eq!(harness.state().note.id, active);
    fixtures::capture(&mut harness, "tab-order-persisted");
    drop(harness);
    let reopened = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(
        reopened
            .tabs
            .iter()
            .map(|note| note.id.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(reopened.note.id, active);
    Ok(())
}

#[test]
fn escape_outside_drop_and_same_slot_leave_order_unchanged_and_close_still_confirms() -> Result<()>
{
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = tab_input::ids(&harness);
    let from = harness.get_by_label("Weekend ideas").rect().center();
    let to = harness
        .get_by_label("A thought worth keeping")
        .rect()
        .left_center();
    tab_input::start(&mut harness, from, to);
    harness.key_press(Key::Escape);
    harness.step();
    assert!(!DragAndDrop::has_any_payload(&harness.ctx));
    tab_input::finish(&mut harness, to);
    assert_eq!(tab_input::ids(&harness), original);
    assert!(harness.get_by_label("Note content").is_focused());

    let outside = harness.get_by_label("Note content").rect().center();
    tab_input::start(&mut harness, from, outside);
    tab_input::finish(&mut harness, outside);
    assert_eq!(tab_input::ids(&harness), original);
    assert!(harness.get_by_label("Note content").is_focused());
    assert!(harness.state().store.setting("tabOrder")?.is_none());

    let nearby = from + egui::vec2(8.0, 0.0);
    tab_input::start(&mut harness, from, nearby);
    tab_input::finish(&mut harness, nearby);
    assert_eq!(tab_input::ids(&harness), original);
    assert!(harness.get_by_label("Note content").is_focused());
    assert!(harness.state().store.setting("tabOrder")?.is_none());
    harness.get_by_label("Delete Weekend ideas").click();
    harness.run();
    assert_eq!(
        harness.state().pending_delete.as_deref(),
        Some(original[2].as_str())
    );
    assert!(harness.get_by_label("Cancel").is_focused());
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(tab_input::ids(&harness), original);
    Ok(())
}

#[test]
fn order_write_failure_keeps_visible_tabs_and_the_editor_usable() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    Connection::open(directory.path().join("notes.sqlite3"))?.execute_batch(
        "CREATE TRIGGER reject_order BEFORE INSERT ON app_state WHEN NEW.key='tabOrder' \
            BEGIN SELECT RAISE(ABORT,'order write rejected'); END;",
    )?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = tab_input::ids(&harness);
    let active = harness.state().note.id.clone();
    let body = harness.state().note.body.clone();
    let from = harness.get_by_label("Weekend ideas").rect().center();
    let to = harness
        .get_by_label("A thought worth keeping")
        .rect()
        .left_center();
    tab_input::start(&mut harness, from, to);
    tab_input::finish(&mut harness, to);
    assert_eq!(tab_input::ids(&harness), original);
    assert!(harness.state().error.is_some());
    assert_eq!(harness.state().note.id, active);
    assert_eq!(harness.state().note.body, body);
    harness.event(Event::Text("still editable".into()));
    harness.run();
    assert!(harness.state().note.body.contains("still editable"));
    assert!(!harness.state().dirty);
    Ok(())
}
