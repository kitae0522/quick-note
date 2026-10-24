use crate::fixtures;
use eframe::egui;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;

#[test]
fn the_whole_note_canvas_accepts_typing_after_focus_leaves_the_editor() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    for (index, position) in [
        egui::pos2(4.0, 100.0),
        egui::pos2(916.0, 100.0),
        egui::pos2(400.0, 550.0),
        egui::pos2(4.0, 550.0),
        egui::pos2(400.0, 44.0),
    ]
    .into_iter()
    .enumerate()
    {
        harness.get_by_label("Search notes").focus();
        harness.run();
        harness.event(Event::PointerMoved(position));
        for pressed in [true, false] {
            harness.event(Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        harness.run();
        assert!(
            harness.get_by_label("Note content").is_focused(),
            "{position:?}"
        );
        let text = format!("[click-{index}]");
        harness.event(Event::Text(text.clone()));
        harness.run();
        assert!(harness.state().note.body.contains(&text));
    }
    Ok(())
}

#[test]
fn the_first_text_in_the_new_note_frame_is_never_dropped() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.input_mut().events = vec![
        Event::Key {
            key: Key::N,
            physical_key: Some(Key::N),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        },
        Event::Text("first".into()),
    ];
    harness.step();
    harness.run();
    assert_eq!(harness.state().note.body, "first");
    Ok(())
}

#[test]
fn korean_preedit_replaces_composition_and_commits_each_syllable_once() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run();
    for text in ["ㅎ", "하", "한"] {
        harness.event(Event::Ime(ImeEvent::Preedit {
            text: text.into(),
            active_range_chars: None,
        }));
        harness.run();
        assert_eq!(harness.state().note.body, text);
    }
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: String::new(),
        active_range_chars: None,
    }));
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    harness.event(Event::Text(" ".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "한 ");
    harness.event(Event::Text("aa  ".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "한 aa  ");
    let state = harness.state();
    assert_eq!(
        state
            .store
            .note(&state.note.id)?
            .expect("saved fixture")
            .body,
        state.note.body
    );
    Ok(())
}

#[test]
fn cut_undo_redo_and_paste_preserve_repeated_unicode_text() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run();
    let text = "한한  aa 💡 café";
    harness.event(Event::Paste(text.into()));
    harness.run();
    assert_eq!(harness.state().note.body, text);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.run();
    harness.event(Event::Cut);
    harness.run();
    assert!(harness.state().note.body.is_empty());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, text);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.run();
    assert!(harness.state().note.body.is_empty());
    Ok(())
}
