use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use egui_kittest::kittest::Queryable;
use quick_note::app::Mode;
use quick_note::app::QuickNoteApp;
use quick_note::i18n::Language;
use quick_note::storage::Result;

#[test]
fn markdown_switch_disables_preview_without_changing_text_or_cursor() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let body = app.note.body.clone();
    let selection = app.note.selection.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    assert_eq!(harness.state().mode, Mode::Preview);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    assert!(
        harness.query_by_label("Markdown").is_some(),
        "Options must expose the Markdown switch"
    );
    harness.get_by_label("Markdown").focus();
    harness.run();
    harness.key_press(Key::Space);
    harness.run();
    assert_eq!(harness.state().mode, Mode::Edit);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.query_by_label("Preview").is_none());
    assert!(harness.get_by_label("Note content").is_focused());
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    assert_eq!(harness.state().mode, Mode::Edit);
    assert_eq!(harness.state().note.body, body);
    assert_eq!(harness.state().note.selection, selection);
    Ok(())
}

#[test]
fn disabled_markdown_persists_and_can_be_enabled_again_without_rewriting_notes() -> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    let body = app.note.body.clone();
    app.set_markdown_enabled(false)?;
    drop(app);
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert!(!app.markdown_enabled());
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    assert!(harness.query_by_label("Preview").is_none());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Markdown").click();
    harness.run();
    assert!(harness.state().markdown_enabled());
    assert_eq!(harness.state().mode, Mode::Edit);
    fixtures::capture(&mut harness, "markdown-switch-on");
    harness.key_press(Key::Escape);
    harness.run();
    harness.get_by_label("Preview").click();
    harness.run();
    assert_eq!(harness.state().mode, Mode::Preview);
    harness.get_by_label("Try next");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Markdown").click();
    harness.run();
    fixtures::capture(&mut harness, "markdown-switch-off");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    assert_eq!(harness.state().note.body, body);
    assert_eq!(
        harness
            .state()
            .store
            .note(&harness.state().note.id)?
            .expect("saved fixture")
            .body,
        body
    );
    Ok(())
}

#[test]
fn korean_switch_is_keyboard_reachable_at_minimum_size() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_language(Language::Korean, &Context::default())?;
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    for _ in 0..5 {
        harness.key_press(Key::Tab);
        harness.run();
    }
    assert!(harness.get_by_label("마크다운").is_focused());
    harness.key_press(Key::Space);
    harness.run();
    assert!(!harness.state().markdown_enabled());
    fixtures::capture(&mut harness, "markdown-switch-narrow-ko");
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    assert_eq!(harness.state().mode, Mode::Edit);
    assert!(harness.get_by_label("노트 내용").is_focused());
    Ok(())
}

#[test]
fn cached_preview_reflects_source_edits_and_window_width_changes() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body = "# Before\n\nA paragraph that wraps as the window changes. 한글 💡".into();
    assert!(app.persist());
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    harness.get_by_role_and_label(Role::Label, "Before");
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.event(Event::Paste("# After\n\n새로운 내용 💡".into()));
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
    harness.run();
    assert!(harness.query_by_label("Before").is_none());
    for size in [[320.0, 240.0], [1440.0, 800.0], [620.0, 400.0]] {
        harness.set_size(size.into());
        harness.run();
        let heading = harness.get_by_role_and_label(Role::Label, "After").rect();
        assert!(heading.left() >= 24.0 && heading.right() <= size[0] - 24.0);
        assert!(heading.top() >= harness.state().theme.layout.toolbar_height + 24.0);
    }
    assert_eq!(
        harness
            .state()
            .store
            .note(&harness.state().note.id)?
            .expect("saved fixture")
            .body,
        "# After\n\n새로운 내용 💡"
    );
    Ok(())
}
