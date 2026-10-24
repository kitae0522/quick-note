use crate::fixtures;
use crate::pair_fixture;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;

#[test]
fn korean_preedit_and_commit_stay_literal_then_normal_pairing_resumes() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("", 0, 0)?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    for text in ["ㅎ", "하", "한"] {
        harness.event(Event::Ime(ImeEvent::Preedit {
            text: text.into(),
            active_range_chars: None,
        }));
        harness.run();
        assert_eq!(harness.state().note.body, text);
    }
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    harness.event(Event::Text("{".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "한{}");
    harness.key_press(Key::Backspace);
    harness.run();
    assert_eq!(harness.state().note.body, "한");
    harness.key_press(Key::Backspace);
    harness.run();
    assert_eq!(harness.state().note.body, "");
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "(".into(),
        active_range_chars: None,
    }));
    harness.run();
    harness.event(Event::Ime(ImeEvent::Commit("(".into())));
    harness.run();
    assert_eq!(harness.state().note.body, "(");
    Ok(())
}

#[test]
fn paste_and_multi_character_text_remain_literal_and_search_does_not_pair() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("", 0, 0)?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    harness.event(Event::Paste("(".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "(");
    harness.event(Event::Paste("{[".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "({[");
    harness.event(Event::Text("{}()[]".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "({[{}()[]");
    let original = harness.state().note.body.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.event(Event::Text("{".into()));
    harness.run();
    assert_eq!(harness.state().header.query, "{");
    assert_eq!(harness.state().note.body, original);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn pairs_follow_canvas_focus_after_search_and_window_reactivation() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("word", 4, 4)?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.input_mut().focused = false;
    harness.event(Event::WindowFocused(false));
    harness.run();
    harness.input_mut().focused = true;
    harness.event(Event::WindowFocused(true));
    harness.run();
    let position = eframe::egui::pos2(918.0, 100.0);
    for pressed in [true, false] {
        harness.event(Event::PointerMoved(position));
        harness.event(Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        harness.run();
    }
    harness.event(Event::Text("{".into()));
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    assert_eq!(harness.state().header.query, "");
    assert_eq!(harness.state().note.body, "word{}");
    Ok(())
}

#[test]
fn the_first_opener_on_a_new_note_pairs_and_stored_unicode_cursor_restores() -> Result<()> {
    let (directory, app) = pair_fixture::editor("old", 3, 3)?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.event(Event::Text("(".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "()");
    harness.event(Event::Text("한💡".into()));
    harness.run();
    let id = harness.state().note.id.clone();
    let cursor = harness.state().note.selection.clone();
    assert_eq!(cursor.location, 4);
    drop(harness);
    let app = quick_note::app::QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(app.note.id, id);
    assert_eq!(app.note.body, "(한💡)");
    assert_eq!(app.note.selection, cursor);
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    harness.event(Event::Text(")".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "(한💡)");
    assert_eq!(harness.state().note.selection.location, 5);
    Ok(())
}
