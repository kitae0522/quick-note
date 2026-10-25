use crate::fixtures;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::appearance;
use quick_note::i18n::Language;
use quick_note::storage::Result;

#[test]
fn preview_uses_the_editor_chain_in_both_languages_without_editing_the_note() -> Result<()> {
    for (language, dark, suffix) in [
        (Language::English, false, "light"),
        (Language::English, true, "dark"),
        (Language::Korean, false, "ko"),
    ] {
        let (_directory, mut app) = fixtures::fixture()?;
        let context = Context::default();
        app.set_language(language, &context)?;
        if dark {
            app.set_theme("graphite", &context)?;
        }
        app.set_editor_fonts(Some(&["Geist Mono".into(), "Pretendard".into()]), &context)?;
        let note = app.note.clone();
        let choose = app.locale.text().choose_font.clone();
        let caption = app.locale.text().font_preview.clone();
        let sample = app.locale.text().font_preview_sample.clone();
        let count = if language == Language::English {
            "2/8 selected"
        } else {
            "2/8개 선택"
        };
        let mut harness = fixtures::harness(app, [920.0, 720.0]);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
        harness.run();
        harness.get_by_label(&choose).click();
        harness.run();
        harness.get_by_label(&caption);
        harness.get_by_label(count);
        let expected = harness.ctx.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(
                    sample.clone(),
                    appearance::editor_font(14.0),
                    Color32::BLACK,
                )
                .size()
                .x
        });
        assert!((harness.get_by_label(&sample).rect().width() - expected).abs() <= 1.0);
        fixtures::capture(&mut harness, &format!("font-polish-{suffix}"));
        harness.key_press(Key::Escape);
        harness.run();
        assert_eq!(harness.state().note.body, note.body);
        assert_eq!(harness.state().note.selection, note.selection);
    }
    Ok(())
}

#[test]
fn selection_limits_and_unavailable_fonts_have_visible_explanations() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
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
        &Context::default(),
    )?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose fonts…").click();
    harness.run();
    harness.get_by_label("8/8 selected");
    harness.get_by_label("8 fonts selected. Remove one to add another.");
    fixtures::capture(&mut harness, "font-polish-limit");
    let context = harness.ctx.clone();
    harness.state_mut().set_editor_fonts(
        Some(&["Pretendard".into(), "Missing Font 108ac".into()]),
        &context,
    )?;
    harness.run();
    harness.get_by_label("Missing Font 108ac · Not installed");
    harness
        .state_mut()
        .set_editor_fonts(Some(&["Pretendard".into()]), &context)?;
    harness.run();
    harness.get_by_label("Keep at least one font in the stack.");
    Ok(())
}
