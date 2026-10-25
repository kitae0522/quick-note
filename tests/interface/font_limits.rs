//! The minimum font selection has a visible and accessible explanation in every layout.
use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use egui_kittest::kittest::NodeT as _;
use egui_kittest::kittest::Queryable;
use quick_note::i18n;
use quick_note::i18n::Language;
use quick_note::storage::Result;

#[test]
fn the_last_font_explains_its_disabled_state_in_both_languages_and_all_layouts() -> Result<()> {
    for (language, zoom) in [(Language::English, 1.0), (Language::Korean, 2.0)] {
        for size in [[920.0, 600.0], [320.0, 600.0], [320.0, 240.0]] {
            let (_directory, mut app) = fixtures::fixture()?;
            let context = Context::default();
            app.set_language(language, &context)?;
            app.set_editor_font(Some("Pretendard"), &context)?;
            let note = app.note.clone();
            let choose = app.locale.text().choose_font.clone();
            let hint = app.locale.text().font_keep_one.clone();
            let source = app.locale.text().font_bundled.clone();
            let order = i18n::format(&app.locale.text().font_order_summary, &[("count", "1")]);
            let remove = i18n::format(&app.locale.text().font_remove, &[("font", "Pretendard")]);
            let mut harness = fixtures::harness(app, [size[0] * zoom, size[1] * zoom]);
            harness.ctx.set_zoom_factor(zoom);
            harness.run();
            harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
            harness.run();
            harness.get_by_label(&choose).focus();
            harness.run();
            harness.key_press(Key::Enter);
            harness.run();
            harness.event(Event::Text("Pretendard".into()));
            harness.run();
            let candidate = harness.get_by_role_and_label(Role::CheckBox, "Pretendard");
            assert!(candidate.accesskit_node().is_disabled());
            assert_eq!(
                candidate.accesskit_node().description().as_deref(),
                Some(hint.as_str()),
                "{language:?}, {size:?}: explain why the last font cannot be unchecked"
            );
            if size[1] < 520.0 {
                harness.get_by_label(&order).focus();
                harness.run();
                harness.key_press(Key::Enter);
                harness.run();
            }
            let label = harness.get_by_label(&hint);
            assert!(
                harness.ctx.content_rect().contains_rect(label.rect()),
                "{language:?}, {size:?}: hint {:?}",
                label.rect()
            );
            let remove_button = harness.get_by_label(&remove).accesskit_node();
            assert!(remove_button.is_disabled());
            assert_eq!(remove_button.description().as_deref(), Some(hint.as_str()));
            if size[1] < 520.0 {
                fixtures::capture(
                    &mut harness,
                    &format!("font-minimum-hint-{}", language.code()),
                );
            }
            let context = harness.ctx.clone();
            harness.state_mut().set_editor_fonts(
                Some(&["Pretendard".into(), "Noto Sans CJK".into()]),
                &context,
            )?;
            harness.run();
            let candidate = harness.get_by_role_and_label(Role::CheckBox, "Pretendard");
            assert!(!candidate.accesskit_node().is_disabled());
            assert_eq!(
                candidate.accesskit_node().description().as_deref(),
                Some(source.as_str())
            );
            let remove_button = harness.get_by_label(&remove).accesskit_node();
            assert!(!remove_button.is_disabled());
            assert_ne!(
                remove_button.description().as_deref(),
                Some(hint.as_str()),
                "enabled removal must not announce the minimum restriction"
            );
            assert!(harness.query_by_label(&hint).is_none());
            assert_eq!(harness.state().note.body, note.body);
            assert_eq!(harness.state().note.selection, note.selection);
        }
    }
    Ok(())
}
