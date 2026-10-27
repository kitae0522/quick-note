use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::app::Mode;
use quick_note::i18n::Language;
use quick_note::storage::Result;

#[test]
fn footer_preview_closes_find_and_replace_like_the_keyboard_command() -> Result<()> {
    for language in [Language::English, Language::Korean] {
        for size in [[920.0, 600.0], [320.0, 240.0]] {
            for replace in [false, true] {
                let (_directory, mut app) = fixtures::fixture()?;
                app.set_language(language, &Context::default())?;
                let text = app.locale.text().clone();
                let body = app.note.body.clone();
                let mut harness = fixtures::harness(app, size);
                harness.run();
                let modifiers = if replace {
                    Modifiers::COMMAND | Modifiers::ALT
                } else {
                    Modifiers::COMMAND
                };
                harness.key_press_modifiers(modifiers, Key::F);
                harness.run();
                harness.event(Event::Text("thought".into()));
                harness.run();
                harness.get_by_label(&text.preview).click();
                harness.run();
                if language == Language::English && size[0] == 920.0 && !replace {
                    fixtures::capture(&mut harness, "find-transition-preview");
                }
                assert_eq!(harness.state().mode, Mode::Preview);
                assert!(harness.query_by_label(&text.find_in_note).is_none());
                assert!(harness.query_by_label(&text.replace_with).is_none());
                assert_eq!(harness.state().note.body, body);
                harness.get_by_label(&text.preview).click();
                harness.run();
                assert_eq!(harness.state().mode, Mode::Preview);
                harness.get_by_label(&text.edit).click();
                harness.run();
                assert!(harness.get_by_label(&text.note_content).is_focused());
                assert_eq!(harness.state().note.body, body);
            }
        }
    }
    Ok(())
}

#[test]
fn header_search_closes_find_and_one_escape_returns_to_writing() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let id = app.note.id.clone();
    let body = app.note.body.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
    harness.run();
    harness.event(Event::Text("thought".into()));
    harness.run();
    harness.get_by_label("Search notes").click();
    harness.run();
    fixtures::capture(&mut harness, "find-transition-header");
    assert!(harness.query_by_label("Find in note").is_none());
    assert!(harness.query_by_label("Replace with").is_none());
    for character in "weekend".chars() {
        harness.event(Event::Text(character.to_string()));
        harness.run();
    }
    assert_eq!(harness.state().header.query, "weekend");
    assert_eq!(harness.state().search_results().len(), 1);
    assert_eq!(harness.state().note.body, body);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.query_by_label("Find a note").is_none());
    assert!(harness.get_by_label("Note content").is_focused());
    harness.event(Event::Text("return-to-note-verified".into()));
    harness.run();
    assert_eq!(harness.state().note.id, id);
    assert_eq!(
        harness
            .state()
            .note
            .body
            .matches("return-to-note-verified")
            .count(),
        1
    );
    assert_eq!(harness.state().header.query, "weekend");
    assert!(!harness.state().dirty);
    Ok(())
}

#[test]
fn command_search_does_not_restore_the_find_field_it_closed() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    assert!(harness.get_by_label("Find in note").is_focused());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    assert!(harness.query_by_label("Find in note").is_none());
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn opening_a_search_result_does_not_treat_the_old_field_focus_as_a_new_request() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness_with_step(app, [920.0, 600.0], 1.0 / 60.0);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.event(Event::Text("weekend".into()));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().note.title(), "Weekend ideas");
    assert!(harness.query_by_label("Find a note").is_none());
    assert!(harness.get_by_label("Note content").is_focused());
    harness.event(Event::Text("result-is-editable".into()));
    harness.run();
    assert!(harness.state().note.body.contains("result-is-editable"));
    assert_eq!(harness.state().header.query, "weekend");
    Ok(())
}
