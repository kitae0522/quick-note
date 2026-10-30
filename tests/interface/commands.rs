use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::storage::Result;
use std::collections::HashSet;

#[test]
fn note_shortcuts_focus_cycle_and_escape_allow_uninterrupted_writing() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num9);
    harness.run();
    assert_eq!(harness.state().note.title(), "Weekend ideas");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    harness.run();
    assert_eq!(harness.state().note.title(), "A thought worth keeping");
    harness.key_press(Key::F6);
    harness.run();
    assert!(harness.get_by_label("A thought worth keeping").is_focused());
    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().note.title(), "Release checklist");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::L);
    harness.run();
    assert_eq!(harness.state().theme.appearance, "dark");
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::L);
    harness.run();
    assert_eq!(harness.state().theme.appearance, "light");
    Ok(())
}

#[test]
fn every_options_action_is_reachable_by_tab_and_help_restores_the_editor() -> Result<()> {
    for size in [[920.0, 600.0], [320.0, 240.0]] {
        let (_directory, app) = fixtures::fixture()?;
        let mut harness = fixtures::harness(app, size);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::P);
        harness.run();
        let labels = [
            "English",
            "한국어",
            "Helium · Light",
            "Graphite · Dark",
            "Choose fonts…",
            "Markdown",
            "Focus mode",
            "Keep on top",
            "Recently deleted",
            "Delete note",
            "Import theme…",
            "Export theme…",
            "Open themes folder",
            "Reload theme",
            "Export note…",
            "Find a note…",
            "Keyboard shortcuts",
            "Back to note",
        ];
        harness.get_by_label("Back to note").focus();
        harness.run();
        let mut seen = HashSet::new();
        for _ in 0..labels.len() + 2 {
            for label in labels {
                if harness.get_by_label(label).is_focused() {
                    seen.insert(label);
                }
            }
            assert!(harness.query_by_label("Note content").is_none());
            if seen.len() == labels.len() {
                break;
            }
            harness.key_press(Key::Tab);
            harness.run();
        }
        assert_eq!(seen.len(), labels.len(), "Tab traversal reached {seen:?}");
        harness.key_press(Key::Escape);
        harness.run();
        assert!(harness.get_by_label("Note content").is_focused());
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
        harness.run();
        harness.get_by_label("Back to settings");
        fixtures::capture(
            &mut harness,
            if size[0] < 400.0 {
                "keyboard-shortcuts-narrow"
            } else {
                "keyboard-shortcuts"
            },
        );
        harness.key_press(Key::Escape);
        harness.run();
        assert!(harness.get_by_label("Note content").is_focused());
    }
    Ok(())
}

#[test]
fn composing_in_search_does_not_open_a_note_when_enter_confirms_hangul() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let original = harness.state().note.id.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "아이디어".into(),
        active_range_chars: None,
    }));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().note.id, original);
    harness.event(Event::Ime(ImeEvent::Commit("아이디어".into())));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().note.title(), "Weekend ideas");
    Ok(())
}

#[test]
fn chrome_controls_share_a_centerline_at_standard_and_minimum_widths() -> Result<()> {
    for size in [[920.0, 600.0], [320.0, 240.0]] {
        let (_directory, app) = fixtures::fixture()?;
        let mut harness = fixtures::harness(app, size);
        harness.run();
        let center = harness.get_by_label("New note").rect().center().y;
        let search = if size[0] >= 620.0 {
            "Search notes"
        } else {
            "Find a note"
        };
        for label in [search, "Open settings", "A thought worth keeping"] {
            let rect = harness.get_by_label(label).rect();
            assert!((rect.center().y - center).abs() <= 1.0, "{label}: {rect:?}");
        }
        let edit = harness.get_by_label("Edit").rect().center().y;
        for label in ["Preview", "Saved", "UTF-8"] {
            let rect = harness.get_by_label(label).rect();
            assert!((rect.center().y - edit).abs() <= 1.0, "{label}: {rect:?}");
        }
    }
    Ok(())
}

#[test]
fn opening_search_from_options_keeps_input_out_of_the_note() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let body = harness.state().note.body.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Find a note…").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.get_by_label("Search notes").is_focused());
    harness.event(Event::Text("Weekend".into()));
    harness.run();
    assert_eq!(harness.state().header.query, "Weekend");
    assert_eq!(harness.state().note.body, body);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}
