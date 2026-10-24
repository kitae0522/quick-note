use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use eframe::egui::ViewportEvent;
use eframe::egui::ViewportId;
use egui_kittest::kittest::Queryable;
use quick_note::app::Mode;
use quick_note::app::QuickNoteApp;
use quick_note::i18n::Language;
use quick_note::storage::Result;
use rusqlite::Connection;

#[test]
fn settings_replaces_the_document_and_command_w_returns_without_deleting_it() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let body = app.note.body.clone();
    let id = app.note.id.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.run();
    let selection = harness.state().note.selection.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    assert!(
        harness.query_by_label("Note content").is_none(),
        "Settings must be a document page, not an overlay"
    );
    assert!(harness.query_by_role(Role::Dialog).is_none());
    harness.get_by_label("Back to note");
    fixtures::capture(&mut harness, "settings-page");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    assert!(harness.state().pending_delete.is_none());
    assert_eq!(harness.state().tabs.len(), 3);
    assert_eq!(harness.state().note.id, id);
    assert_eq!(harness.state().note.body, body);
    assert_eq!(harness.state().note.selection, selection);
    Ok(())
}

#[test]
fn settings_header_opens_a_page_and_back_preserves_preview_in_both_languages() -> Result<()> {
    for (language, size, dark) in [
        (Language::English, [920.0, 600.0], false),
        (Language::Korean, [320.0, 240.0], false),
        (Language::English, [920.0, 600.0], true),
    ] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        if dark {
            app.set_theme("graphite", &Context::default())?;
        }
        let open = app.locale.text().more_options.clone();
        let back = app.locale.text().back_to_note.clone();
        let title = app.locale.text().options.clone();
        let body = app.note.body.clone();
        let mut harness = fixtures::harness(app, size);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
        harness.run();
        assert_eq!(harness.state().mode, Mode::Preview);
        harness.get_by_label(&open).click();
        harness.run();
        assert!(harness.state().settings_open());
        assert!(harness.query_by_role(Role::Dialog).is_none());
        harness.get_by_label(&title);
        let rect = harness.get_by_label(&back).rect();
        assert!(rect.left() >= 0.0 && rect.right() <= size[0]);
        assert!(rect.top() >= 40.0 && rect.bottom() <= size[1]);
        fixtures::capture(
            &mut harness,
            if dark {
                "settings-page-dark"
            } else if language == Language::Korean {
                "settings-page-narrow-ko"
            } else {
                "settings-page-from-preview"
            },
        );
        harness.get_by_label(&back).click();
        harness.run();
        assert!(!harness.state().settings_open());
        assert_eq!(harness.state().mode, Mode::Preview);
        assert_eq!(harness.state().note.body, body);
    }
    Ok(())
}

#[test]
fn closing_settings_keeps_even_an_empty_note_in_place() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let id = app.note.id.clone();
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert!(!harness.state().settings_open());
    assert_eq!(harness.state().note.id, id);
    assert_eq!(harness.state().tabs.len(), 1);
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn failed_close_from_settings_focuses_keep_editing_and_returns_to_the_draft() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let connection = Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT, 'settings close fixture'); END;")?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.event(Event::Text("UNSAVED ".into()));
    harness.run();
    let body = harness.state().note.body.clone();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness
        .input_mut()
        .viewports
        .get_mut(&ViewportId::ROOT)
        .expect("root viewport")
        .events
        .push(ViewportEvent::Close);
    harness.run();
    let dialog = harness.get_by_role(Role::Dialog);
    assert!(dialog.get_by_label("Keep editing").is_focused());
    harness.key_press(Key::Enter);
    harness.run();
    assert!(!harness.state().settings_open());
    assert!(harness.get_by_label("Note content").is_focused());
    assert!(harness.state().dirty);
    assert_eq!(harness.state().note.body, body);
    Ok(())
}
