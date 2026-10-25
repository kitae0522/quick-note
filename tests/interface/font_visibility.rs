//! Font controls remain reachable when the viewport cannot fit two independent lists.
use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::MouseWheelSource;
use eframe::egui::MouseWheelUnit;
use eframe::egui::Role;
use eframe::egui::TouchPhase;
use egui_kittest::Harness;
use egui_kittest::kittest::NodeT as _;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::i18n;
use quick_note::i18n::Language;
use quick_note::storage::Result;

fn open_fonts(harness: &mut Harness<'_, QuickNoteApp>, choose: &str) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label(choose).focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
}

#[test]
fn resizing_reveals_the_focused_candidate_without_overriding_manual_scrolling() -> Result<()> {
    for (language, zoom) in [(Language::English, 1.0), (Language::Korean, 2.0)] {
        let (_directory, mut app) = fixtures::fixture()?;
        let context = Context::default();
        app.set_language(language, &context)?;
        app.set_editor_font(Some("monospace"), &context)?;
        let choose = app.locale.text().choose_font.clone();
        let note = app.note.clone();
        let mut harness = fixtures::harness(app, [920.0 * zoom, 600.0 * zoom]);
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        open_fonts(&mut harness, &choose);
        let name = harness
            .query_all_by_role(Role::CheckBox)
            .rfind(|node| !node.accesskit_node().is_disabled())
            .expect("the catalog includes available bundled families")
            .accesskit_node()
            .label()
            .expect("font candidates have accessible names");
        harness.get_by_role_and_label(Role::CheckBox, &name).focus();
        harness.run();
        for size in [[320.0, 240.0], [320.0, 600.0], [920.0, 600.0]] {
            harness.set_size(eframe::egui::vec2(size[0] * zoom, size[1] * zoom));
            harness.run();
            let candidate = harness.get_by_role_and_label(Role::CheckBox, &name);
            assert!(candidate.is_focused());
            assert!(
                harness.ctx.content_rect().contains_rect(candidate.rect()),
                "{size:?}: focused {name} is outside the window at {:?}",
                candidate.rect()
            );
        }
        harness.key_press(Key::Enter);
        harness.run();
        assert_eq!(
            harness.state().theme.typography.font_fallbacks,
            [name.as_str()]
        );
        let position = harness
            .get_by_role_and_label(Role::CheckBox, &name)
            .rect()
            .center();
        harness.hover_at(position);
        harness.event(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            source: MouseWheelSource::Wheel,
            phase: TouchPhase::Move,
            delta: eframe::egui::vec2(0.0, 3600.0),
            modifiers: Modifiers::NONE,
        });
        harness.run();
        let candidate = harness.get_by_role_and_label(Role::CheckBox, &name);
        assert!(candidate.is_focused());
        assert!(
            !harness.ctx.content_rect().contains_rect(candidate.rect()),
            "manual scrolling must not snap back to the focused candidate"
        );
        assert_eq!(harness.state().note.body, note.body);
        assert_eq!(harness.state().note.selection, note.selection);
    }
    Ok(())
}

#[test]
fn full_stacks_keep_candidates_and_order_actions_reachable_at_minimum_size() -> Result<()> {
    for (language, zoom) in [(Language::English, 1.0), (Language::Korean, 2.0)] {
        for header_height in [44.0, 56.0] {
            let (_directory, mut app) = fixtures::fixture()?;
            let context = Context::default();
            app.set_language(language, &context)?;
            app.theme.layout.toolbar_height = header_height;
            app.theme.validate()?;
            app.set_editor_fonts(
                Some(&[
                    "monospace".into(),
                    "Pretendard".into(),
                    "Noto Sans CJK".into(),
                    "Geist".into(),
                    "Geist Mono".into(),
                    "Geist Medium".into(),
                    "Geist Bold".into(),
                    "Noto Emoji".into(),
                ]),
                &context,
            )?;
            let choose = app.locale.text().choose_font.clone();
            let order = i18n::format(&app.locale.text().font_order_summary, &[("count", "8")]);
            let move_up = i18n::format(
                &app.locale.text().font_move_up,
                &[("font", "Noto Sans CJK")],
            );
            let note = app.note.clone();
            let mut harness = fixtures::harness(app, [320.0 * zoom, 240.0 * zoom]);
            harness.ctx.set_zoom_factor(zoom);
            harness.run();
            open_fonts(&mut harness, &choose);
            harness.get_by_label(&order).focus();
            harness.run();
            harness.key_press(Key::Enter);
            harness.run();
            harness
                .get_by_role_and_label(Role::CheckBox, "Pretendard")
                .focus();
            harness.run();
            let candidate = harness.get_by_role_and_label(Role::CheckBox, "Pretendard");
            assert!(candidate.is_focused());
            assert!(
                harness.ctx.content_rect().contains_rect(candidate.rect()),
                "{language:?}, header {header_height}: candidate {:?}",
                candidate.rect()
            );
            fixtures::capture(
                &mut harness,
                &format!("font-full-stack-{}-{header_height}", language.code()),
            );
            harness.get_by_label(&move_up).focus();
            harness.run();
            let action = harness.get_by_label(&move_up);
            assert!(action.is_focused());
            assert!(harness.ctx.content_rect().contains_rect(action.rect()));
            harness.key_press(Key::Enter);
            harness.run();
            assert_eq!(
                harness.state().theme.typography.font_fallbacks[0],
                "Noto Sans CJK"
            );
            assert_eq!(harness.state().note.body, note.body);
            assert_eq!(harness.state().note.selection, note.selection);
        }
    }
    Ok(())
}
