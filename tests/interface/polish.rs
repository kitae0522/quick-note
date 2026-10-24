//! Accessible preference controls, zoom, keyboard paths and failed-write protection.
use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use egui_kittest::kittest::NodeT as _;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::i18n::Language;
use quick_note::storage::Result;
use rusqlite::Connection;

#[test]
fn motion_preference_is_keyboard_operable_and_persists_without_rewriting_notes() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let original = app.note.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    let label = harness.state().locale.text().reduce_motion.clone();
    let switch = harness.get_by_role_and_label(Role::Switch, &label);
    if switch.accesskit_node().is_disabled() {
        assert!(harness.state().reduced_motion());
        return Ok(()); // The system preference may already require reduced motion.
    }
    switch.focus();
    harness.run();
    assert!(harness.get_by_label(&label).is_focused());
    harness.key_press(Key::Space);
    harness.run();
    assert!(harness.state().reduced_motion());
    assert_eq!(
        harness.ctx.style_of(harness.ctx.theme()).animation_time,
        0.0
    );
    assert_eq!(harness.state().note.body, original.body);
    assert_eq!(harness.state().note.selection, original.selection);
    let stored = harness
        .state()
        .store
        .note(&original.id)?
        .expect("fixture note");
    assert_eq!(stored.updated_at, original.updated_at);
    assert_eq!(stored.body, original.body);
    drop(harness);
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert!(app.reduced_motion());
    assert_eq!(app.store.setting("reduceMotion")?.as_deref(), Some("true"));
    Ok(())
}

#[test]
fn failed_motion_write_preserves_the_preference_and_exposes_a_recoverable_alert() -> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    let context = Context::default();
    app.set_reduced_motion(true, &context)?;
    let database = Connection::open(directory.path().join("notes.sqlite3"))?;
    database.execute_batch("CREATE TRIGGER fail_motion BEFORE INSERT ON app_state BEGIN SELECT RAISE(ABORT, 'motion fixture'); END;")?;
    assert!(app.set_reduced_motion(false, &context).is_err());
    assert!(app.reduced_motion());
    assert_eq!(app.store.setting("reduceMotion")?.as_deref(), Some("true"));
    app.error = Some(
        quick_note::i18n::message(|m| &m.error_motion_setting)
            .cause("error", "motion fixture")
            .into(),
    );
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let alert = harness.get_by_role(Role::Alert);
    assert!(
        alert
            .accesskit_node()
            .label()
            .expect("error label")
            .contains("Try again")
    );
    assert_eq!(
        alert.accesskit_node().live(),
        eframe::egui::accesskit::Live::Assertive
    );
    harness.get_by_label("Dismiss").click();
    harness.run();
    assert!(harness.state().error.is_none());
    assert_eq!(
        harness.get_by_role(Role::Status).accesskit_node().live(),
        eframe::egui::accesskit::Live::Polite
    );
    Ok(())
}

#[test]
fn controls_keep_names_targets_and_safe_keyboard_paths_at_200_percent_zoom() -> Result<()> {
    for language in [Language::English, Language::Korean] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        app.set_reduced_motion(true, &Context::default())?;
        let mut harness = fixtures::harness(app, [640.0, 480.0]);
        harness.ctx.set_zoom_factor(2.0);
        harness.run();
        let size = harness.ctx.content_rect();
        assert!((size.width() - 320.0).abs() < 1.0);
        for label in [
            harness.state().locale.text().new_note.as_str(),
            harness.state().locale.text().more_options.as_str(),
        ] {
            let node = harness.get_by_label(label);
            assert!(size.contains_rect(node.rect()));
            assert!(node.rect().width() >= 24.0 && node.rect().height() >= 24.0);
        }
        harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
        harness.run();
        assert!(
            harness
                .get_by_label(&harness.state().locale.text().search_notes)
                .is_focused()
        );
        harness.key_press(Key::Escape);
        harness.run();
        assert!(
            harness
                .get_by_label(&harness.state().locale.text().note_content)
                .is_focused()
        );
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
        harness.run();
        let back = harness.get_by_label(&harness.state().locale.text().back_to_note);
        assert!(size.contains_rect(back.rect()));
        for role in [Role::Button, Role::Switch, Role::TextInput] {
            for node in harness.query_all_by_role(role) {
                assert!(
                    node.accesskit_node()
                        .label()
                        .is_some_and(|name| !name.trim().is_empty()),
                    "unnamed control: {node:?}"
                );
            }
        }
        harness.key_press(Key::Escape);
        harness.run();
        assert!(!harness.state().settings_open());
        fixtures::capture(&mut harness, &format!("polish-zoom-{}", language.code()));
    }
    Ok(())
}

#[test]
fn text_navigation_and_cancel_work_during_the_first_animation_frame() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness_with_step(app, [920.0, 600.0], 1.0 / 60.0);
    harness.run();
    let target = harness
        .state()
        .tabs
        .iter()
        .find(|tab| tab.title == "Release checklist")
        .expect("fixture note");
    let expected = harness
        .state()
        .store
        .note(&target.id)?
        .expect("stored fixture note")
        .body;
    quick_note::ui::motion::configure(&harness.ctx, false);
    harness.step();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.step();
    assert!(harness.get_by_label("Search notes").is_focused());
    harness.event(Event::Text("Release".into()));
    harness.step();
    harness.key_press(Key::Enter);
    harness.step();
    assert_eq!(harness.state().note.title(), "Release checklist");
    assert_eq!(harness.state().note.body, expected);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.step();
    assert!(harness.get_by_label("Cancel").is_focused());
    harness.key_press(Key::Enter);
    harness.step();
    assert!(harness.state().pending_delete.is_none());
    assert_eq!(harness.state().tabs.len(), 3);
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn compact_search_has_a_visible_two_pixel_keyboard_focus_indicator() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    let search = harness.get_by_label("Search notes");
    assert!(search.is_focused());
    let rect = search.rect();
    let accent = harness.state().theme.palette().accent.to_array();
    let image = harness.render()?;
    let mut colored = 0;
    for y in rect.top().ceil() as u32..rect.bottom().floor() as u32 {
        for x in rect.left().ceil() as u32..rect.right().floor() as u32 {
            if image.get_pixel(x, y).0 == accent {
                colored += 1;
            }
        }
    }
    assert!(
        colored as f32 > 3.0 * (rect.width() + rect.height()),
        "focus needs a visible perimeter, measured {colored} accent pixels around {rect:?}"
    );
    fixtures::capture(&mut harness, "polish-compact-focus");
    Ok(())
}
