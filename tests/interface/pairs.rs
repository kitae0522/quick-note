use crate::fixtures;
use crate::pair_fixture;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::model::Selection;
use quick_note::storage::Result;

#[test]
fn opening_pairs_place_the_cursor_inside_and_backspace_removes_both() -> Result<()> {
    for (opening, pair) in [("(", "()"), ("[", "[]"), ("{", "{}")] {
        let (_directory, app) = pair_fixture::editor("", 0, 0)?;
        let mut harness = fixtures::harness(app, [640.0, 400.0]);
        harness.run();
        harness.event(Event::Text(opening.into()));
        harness.run();
        assert_eq!(harness.state().note.body, pair);
        assert_eq!(
            harness.state().note.selection,
            Selection {
                location: 1,
                length: 0
            }
        );
        assert!(harness.get_by_label("Note content").is_focused());
        harness.key_press(Key::Backspace);
        harness.run();
        assert_eq!(harness.state().note.body, "");
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
        harness.run();
        assert_eq!(harness.state().note.body, pair);
        assert_eq!(harness.state().note.selection.location, 1);
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
        harness.run();
        assert_eq!(harness.state().note.body, "");
    }
    Ok(())
}

#[test]
fn ordered_fast_input_nests_pairs_and_skips_the_existing_closers() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("", 0, 0)?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    for text in ["(", "{", "[", "한글 💡", "]", "}", ")"] {
        harness.event(Event::Text(text.into()));
    }
    harness.run();
    let body = "({[한글 💡]})";
    assert_eq!(harness.state().note.body, body);
    assert_eq!(
        harness.state().note.selection,
        Selection::from_char_range(body, body.chars().count(), body.chars().count())
    );
    let state = harness.state();
    assert_eq!(
        state.store.note(&state.note.id)?.expect("saved draft").body,
        body
    );
    Ok(())
}

#[test]
fn wrapping_unicode_and_multiline_selections_preserves_selection_and_atomic_undo() -> Result<()> {
    let original = "한글 💡\ne\u{301}";
    let (_directory, app) = pair_fixture::editor(original, 0, original.chars().count())?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    for (opening, expected, offset) in [
        ("{", format!("{{{original}}}"), 1),
        ("(", format!("{{({original})}}"), 2),
    ] {
        harness.event(Event::Text(opening.into()));
        harness.run();
        assert_eq!(harness.state().note.body, expected);
        assert_eq!(
            harness.state().note.selection,
            Selection {
                location: offset,
                length: original.encode_utf16().count()
            }
        );
    }
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, format!("{{{original}}}"));
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, format!("{{({original})}}"));
    fixtures::capture(&mut harness, "paired-unicode-selection");
    Ok(())
}

#[test]
fn selection_navigation_and_wrapping_in_one_batch_use_the_current_cursor() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("word", 0, 0)?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    for _ in 0..4 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowRight);
    }
    harness.event(Event::Text("{".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "{word}");
    assert_eq!(
        harness.state().note.selection,
        Selection {
            location: 1,
            length: 4
        }
    );
    harness.event(Event::Text("new".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "{new}");
    Ok(())
}

#[test]
fn reversed_selection_wraps_in_place_and_opening_a_pair_undoes_atomically() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("word", 4, 4)?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    for _ in 0..4 {
        harness.key_press_modifiers(Modifiers::SHIFT, Key::ArrowLeft);
    }
    harness.event(Event::Text("[".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "[word]");
    harness.key_press(Key::ArrowLeft);
    harness.event(Event::Text("(".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "[()word]");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, "[word]");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, "word");
    Ok(())
}

#[test]
fn overtyping_moves_only_the_cursor_and_mismatched_closers_insert_normally() -> Result<()> {
    let (_directory, app) = pair_fixture::editor("()", 1, 1)?;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    let revision = harness.state().note.updated_at;
    harness.event(Event::Text(")".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "()");
    assert_eq!(harness.state().note.selection.location, 2);
    assert_eq!(harness.state().note.updated_at, revision);
    harness.key_press(Key::ArrowLeft);
    harness.event(Event::Text("]".into()));
    harness.run();
    assert_eq!(harness.state().note.body, "(])");
    harness.key_press(Key::Backspace);
    harness.run();
    assert_eq!(harness.state().note.body, "()");
    Ok(())
}
