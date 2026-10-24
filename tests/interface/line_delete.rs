//! Line deletion uses one key event, including after a composition commit in the same batch.
use crate::fixtures;
use crate::pair_fixture;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use quick_note::storage::Result;

const COMMAND: Modifiers = Modifiers {
    mac_cmd: true,
    command: true,
    ..Modifiers::NONE
};

#[test]
fn command_backspace_deletes_once_and_undo_restores_the_unicode_line() -> Result<()> {
    let body = "Keep this\nPrefix 한글 💡 café";
    let end = body.chars().count();
    let (_directory, app) = pair_fixture::editor(body, end, end)?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(COMMAND, Key::Backspace);
    harness.run();
    assert_eq!(harness.state().note.body, "Keep this\n");
    assert_eq!(harness.state().note.selection.location, 10);
    assert_eq!(harness.state().note.selection.length, 0);
    assert_eq!(
        harness
            .state()
            .store
            .note(&harness.state().note.id)?
            .expect("saved draft")
            .body,
        "Keep this\n"
    );
    harness.key_press_modifiers(COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, body);
    Ok(())
}

#[test]
fn commit_then_command_backspace_in_one_batch_deletes_the_whole_line_once() -> Result<()> {
    let prefix = "Keep this\nPrefix ";
    let end = prefix.chars().count();
    let (_directory, app) = pair_fixture::editor(prefix, end, end)?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "한".into(),
        active_range_chars: None,
    }));
    harness.run();
    assert_eq!(harness.state().note.body, format!("{prefix}한"));
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    harness.key_press_modifiers(COMMAND, Key::Backspace);
    harness.run();
    assert_eq!(harness.state().note.body, "Keep this\n");
    assert!(!harness.state().dirty);
    fixtures::capture(&mut harness, "command-backspace-after-commit");
    harness.key_press_modifiers(COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, format!("{prefix}한"));
    Ok(())
}

#[test]
fn ordinary_composition_corrections_do_not_delete_the_committed_prefix() -> Result<()> {
    let prefix = "Keep this\nPrefix ";
    let end = prefix.chars().count();
    let (_directory, app) = pair_fixture::editor(prefix, end, end)?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    for text in ["한", "하", "ㅎ", ""] {
        harness.event(Event::Ime(ImeEvent::Preedit {
            text: text.into(),
            active_range_chars: None,
        }));
        harness.run();
        assert_eq!(harness.state().note.body, format!("{prefix}{text}"));
    }
    Ok(())
}

#[test]
fn command_backspace_in_composing_search_does_not_touch_the_note() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("Keep this note", 14, 14)?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(COMMAND, Key::P);
    harness.run();
    harness.event(Event::Text("Prefix ".into()));
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "한".into(),
        active_range_chars: None,
    }));
    harness.run();
    assert_eq!(harness.state().header.query, "Prefix 한");
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    harness.key_press_modifiers(COMMAND, Key::Backspace);
    harness.run();
    assert!(harness.state().header.query.is_empty());
    assert_eq!(harness.state().note.body, "Keep this note");
    harness.key_press_modifiers(COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().header.query, "Prefix 한");
    assert_eq!(harness.state().note.body, "Keep this note");
    Ok(())
}
