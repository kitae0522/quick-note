//! Large collections retain complete navigation while creating only visible widgets.
use crate::fixtures;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use quick_note::storage::Store;
use rusqlite::Connection;
use rusqlite::params;

#[test]
fn a_thousand_tabs_and_results_remain_reachable_by_keyboard() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("notes.sqlite3");
    drop(Store::try_new(&path)?);
    let mut connection = Connection::open(&path)?;
    let transaction = connection.transaction()?;
    for index in 0..1000 {
        transaction.execute(
            "INSERT INTO notes(id,body,title,created_at,updated_at) VALUES(?1,?2,?2,?3,?3)",
            params![index.to_string(), format!("Idea {index:04}"), index],
        )?;
    }
    transaction.execute("INSERT INTO app_state VALUES('activeNote','0')", [])?;
    transaction.commit()?;
    drop(connection);
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    assert!(harness.get_all_by_role(Role::Tab).count() < 10);
    harness.key_press(Key::F6);
    harness.run();
    harness.key_press(Key::End);
    harness.run();
    assert_eq!(harness.state().note.id, "999");
    assert!(harness.get_by_label("Idea 0999").is_focused());
    let close = harness.get_by_label("Delete Idea 0999").rect();
    assert!(close.right() <= harness.get_by_label("New note").rect().left());
    harness.key_press(Key::Home);
    harness.run();
    assert_eq!(harness.state().note.id, "0");
    assert!(harness.get_by_label("Idea 0000").is_focused());

    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    assert_eq!(harness.state().search_results().len(), 1000);
    assert!(harness.get_all_by_label_contains("Open Idea").count() < 20);
    for _ in 0..20 {
        harness.key_press(Key::ArrowDown);
        harness.run();
    }
    harness.get_by_label("Open Idea 0979");
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().note.id, "979");
    assert!(harness.get_by_label("Note content").is_focused());
    assert_eq!(harness.state().store.summaries()?.len(), 1000);
    Ok(())
}

#[test]
fn normalized_search_updates_after_edits_creates_and_deletions() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.header.query = "CAFÉ".into();
    app.refresh_search()?;
    assert_eq!(app.search_results().len(), 1);
    let original = app.note.body.clone();
    app.note.body.push_str("\nCafé 한글 💡");
    assert!(app.persist());
    app.refresh_search()?;
    assert_eq!(app.search_results().len(), 2);
    app.note.body = original;
    assert!(app.persist());
    app.refresh_search()?;
    assert_eq!(app.search_results().len(), 1);
    app.new_note()?;
    app.note.body = "new CAFÉ".into();
    assert!(app.persist());
    let created = app.note.id.clone();
    app.refresh_search()?;
    assert_eq!(app.search_results().len(), 2);
    app.request_delete(&created)?;
    app.confirm_delete()?;
    app.refresh_search()?;
    assert_eq!(app.search_results().len(), 1);
    assert_eq!(app.search_results()[0].title, "Weekend ideas");
    Ok(())
}
