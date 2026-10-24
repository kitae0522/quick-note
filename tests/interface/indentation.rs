use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;

#[test]
fn tab_inserts_spaces_and_shift_tab_unindents_without_leaving_the_editor() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run();
    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    assert_eq!(harness.state().note.body, "    ");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, "");
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, "    ");
    harness.event(Event::Text("한글 💡".into()));
    harness.run();
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert_eq!(harness.state().note.body, "한글 💡");
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press(Key::Tab);
    harness.run();
    assert_eq!(harness.state().note.body, "한글 💡    ");
    let state = harness.state();
    assert_eq!(
        state.store.note(&state.note.id)?.expect("saved draft").body,
        state.note.body
    );
    Ok(())
}

#[test]
fn f6_exits_editing_while_tab_still_navigates_header_settings_and_search() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let body = harness.state().note.body.clone();
    harness.key_press(Key::F6);
    harness.run();
    assert!(harness.get_by_label("A thought worth keeping").is_focused());
    harness.key_press(Key::Tab);
    harness.run();
    assert!(!harness.get_by_label("Note content").is_focused());
    assert_eq!(harness.state().note.body, body);
    harness.key_press_modifiers(Modifiers::SHIFT, Key::F6);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    assert_ne!(harness.state().note.body, body);
    let edited = harness.state().note.body.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    assert!(harness.get_by_label("English").is_focused());
    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label("한국어").is_focused());
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    assert!(harness.get_by_label("Search notes").is_focused());
    harness.key_press(Key::Tab);
    harness.run();
    assert!(!harness.get_by_label("Search notes").is_focused());
    assert_eq!(harness.state().header.query, "");
    assert_eq!(harness.state().note.body, edited);
    Ok(())
}

#[test]
fn the_first_tab_and_hangul_commit_in_an_input_batch_are_not_lost_or_duplicated() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.input_mut().events = vec![
        Event::Key {
            key: Key::N,
            physical_key: Some(Key::N),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        },
        Event::Key {
            key: Key::Tab,
            physical_key: Some(Key::Tab),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        },
        Event::Text("idea".into()),
    ];
    harness.step();
    harness.run();
    assert_eq!(harness.state().note.body, "    idea");
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run();
    for text in ["ㅎ", "하", "한"] {
        harness.event(Event::Ime(ImeEvent::Preedit {
            text: text.into(),
            active_range_chars: None,
        }));
        harness.run();
    }
    harness.input_mut().events = vec![
        Event::Ime(ImeEvent::Commit("한".into())),
        Event::Key {
            key: Key::Tab,
            physical_key: Some(Key::Tab),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        },
        Event::Text("글".into()),
    ];
    harness.step();
    harness.run();
    assert_eq!(harness.state().note.body, "한    글");
    assert!(harness.get_by_label("Note content").is_focused());
    let state = harness.state();
    assert_eq!(
        state
            .store
            .note(&state.note.id)?
            .expect("saved UTF-8 fixture")
            .body,
        "한    글"
    );
    Ok(())
}
