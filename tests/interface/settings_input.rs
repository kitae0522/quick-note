use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;

#[test]
fn opening_settings_finishes_the_same_frame_korean_commit_once() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let original = app.note.body.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "ㅎ".into(),
        active_range_chars: None,
    }));
    harness.run();
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    assert!(harness.state().settings_open());
    assert_eq!(harness.state().note.body, format!("한{original}"));
    assert_eq!(
        harness
            .state()
            .store
            .note(&harness.state().note.id)?
            .expect("saved note")
            .body,
        harness.state().note.body
    );
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    harness.event(Event::Text("글".into()));
    harness.run();
    assert_eq!(harness.state().note.body, format!("한글{original}"));
    Ok(())
}

#[test]
fn new_note_from_settings_receives_its_first_text_without_changing_the_previous_note() -> Result<()>
{
    let (_directory, app) = fixtures::fixture()?;
    let original_id = app.note.id.clone();
    let original_body = app.note.body.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.event(Event::Text("First idea 아이디어".into()));
    harness.run();
    assert!(!harness.state().settings_open());
    assert_ne!(harness.state().note.id, original_id);
    assert_eq!(harness.state().note.body, "First idea 아이디어");
    assert_eq!(
        harness
            .state()
            .store
            .note(&original_id)?
            .expect("previous note")
            .body,
        original_body
    );
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}
