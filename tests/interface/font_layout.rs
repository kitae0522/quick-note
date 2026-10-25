//! Resizing preserves keyboard work in the selected-font section.
use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::i18n;
use quick_note::i18n::Language;
use quick_note::storage::Result;

fn open_fonts(harness: &mut Harness<'_, QuickNoteApp>, label: &str) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label(label).focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
}

#[test]
fn resizing_preserves_and_reveals_the_focused_font_action_in_both_languages() -> Result<()> {
    for (language, zoom) in [(Language::English, 1.0), (Language::Korean, 2.0)] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        app.set_editor_fonts(
            Some(&[
                "monospace".into(),
                "Pretendard".into(),
                "Noto Sans CJK".into(),
            ]),
            &Context::default(),
        )?;
        let body = app.note.body.clone();
        let choose = app.locale.text().choose_font.clone();
        let action = i18n::format(
            &app.locale.text().font_move_up,
            &[("font", "Noto Sans CJK")],
        );
        let mut harness = fixtures::harness(app, [920.0 * zoom, 600.0 * zoom]);
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        open_fonts(&mut harness, &choose);
        harness.get_by_label(&action).focus();
        harness.run();
        for height in [600.0, 240.0] {
            harness.set_size(eframe::egui::vec2(320.0 * zoom, height * zoom));
            harness.run();
            assert!(
                harness
                    .query_by_label(&action)
                    .is_some_and(|node| node.is_focused()),
                "resizing to 320×{height} must retain {action}"
            );
            assert!(
                harness
                    .ctx
                    .content_rect()
                    .contains_rect(harness.get_by_label(&action).rect())
            );
        }
        fixtures::capture(
            &mut harness,
            &format!("font-resize-focus-{}", language.code()),
        );
        harness.key_press(Key::Enter);
        harness.run();
        assert_eq!(
            harness.state().theme.typography.font_families(),
            ["monospace", "Noto Sans CJK", "Pretendard"]
        );
        harness.set_size(eframe::egui::vec2(920.0 * zoom, 600.0 * zoom));
        harness.run();
        assert!(harness.get_by_label(&action).is_focused());
        assert_eq!(harness.state().note.body, body);
    }
    Ok(())
}

#[test]
fn reset_focus_survives_resize_and_manual_collapse_does_not_steal_focus() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_editor_fonts(
        Some(&["monospace".into(), "Pretendard".into()]),
        &Context::default(),
    )?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    open_fonts(&mut harness, "Choose fonts…");
    harness.set_size(eframe::egui::vec2(320.0, 240.0));
    harness.run();
    harness.get_by_label("Font order · 2/8").focus();
    harness.run();
    harness.set_size(eframe::egui::vec2(920.0, 600.0));
    harness.run();
    assert!(
        harness
            .ctx
            .memory(|memory| memory.has_focus(Id::unique("font-search")))
    );
    harness.get_by_label("Use theme font").focus();
    harness.run();
    harness.set_size(eframe::egui::vec2(320.0, 240.0));
    harness.run();
    assert!(harness.get_by_label("Use theme font").is_focused());
    let header = harness.get_by_label("Font order · 2/8").rect().center();
    harness.event(Event::PointerMoved(header));
    harness.run();
    for pressed in [true, false] {
        harness.event(Event::PointerButton {
            pos: header,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        harness.run();
    }
    assert!(harness.query_by_label("Move Pretendard up").is_none());
    harness.get_by_label("Font order · 2/8").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.query_by_label("Move Pretendard up").is_some());
    harness.key_press(Key::Enter);
    harness.run();
    assert!(harness.query_by_label("Move Pretendard up").is_none());
    assert!(harness.get_by_label("Font order · 2/8").is_focused());
    harness.set_size(eframe::egui::vec2(920.0, 600.0));
    harness.run();
    harness.get_by_label("Move Pretendard up").focus();
    harness.run();
    harness.set_size(eframe::egui::vec2(320.0, 240.0));
    harness.run();
    assert!(harness.get_by_label("Move Pretendard up").is_focused());
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["monospace", "Pretendard"]
    );
    Ok(())
}
