use eframe::egui::Color32;
use eframe::egui::ColorImage;
use eframe::egui::Context;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::RawInput;
use eframe::egui::TextFormat;
use eframe::egui::epaint::text::Glyph;
use eframe::egui::text::LayoutJob;
use quick_note::app::QuickNoteApp;
use quick_note::appearance;
use quick_note::appearance::FontCatalog;
use quick_note::appearance::Theme;
use quick_note::storage::Result;
use rusqlite::Connection;

fn width(names: &[&str], text: &str) -> Result<f32> {
    let mut catalog = FontCatalog::default();
    let names: Vec<_> = names.iter().map(|name| (*name).to_owned()).collect();
    let context = Context::default();
    context.set_fonts(catalog.definitions_for_stack(&names)?);
    let mut width = 0.0;
    let mut output = context.run_ui(RawInput::default(), |ui| {
        width = ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.into(), appearance::editor_font(18.0), Color32::BLACK)
                .size()
                .x
        });
    });
    output.textures_delta.clear();
    Ok(width)
}

fn glyph_pixels(image: &ColorImage, glyph: &Glyph) -> Vec<Color32> {
    let [left, top] = glyph.uv_rect.min.map(usize::from);
    let [right, bottom] = glyph.uv_rect.max.map(usize::from);
    (top..bottom)
        .flat_map(|row| {
            image.pixels[row * image.width() + left..row * image.width() + right]
                .iter()
                .copied()
        })
        .collect()
}

#[test]
fn configured_fallback_order_changes_the_actual_rendered_font() -> Result<()> {
    let text = "iiiiWWWW";
    let geist = width(&["Geist"], text)?;
    let pretendard = width(&["Pretendard"], text)?;
    assert!((geist - pretendard).abs() > 0.2);
    assert!((width(&["Noto Emoji", "Geist", "Pretendard"], text)? - geist).abs() < 0.01);
    assert!((width(&["Noto Emoji", "Pretendard", "Geist"], text)? - pretendard).abs() < 0.01);
    assert!((width(&["Missing face 892a", "Noto Emoji", "Geist"], text)? - geist).abs() < 0.01);
    Ok(())
}

#[test]
fn explicit_fallbacks_keep_the_editor_grid_without_changing_ui_fonts() -> Result<()> {
    let mut catalog = FontCatalog::default();
    let definitions = catalog.definitions_for_stack(&["monospace".into(), "Pretendard".into()])?;
    assert!(
        definitions.font_data["Pretendard"]
            .tweak
            .monospace_cell_width
            .is_none()
    );
    let stack = &definitions.families[&FontFamily::Name("editor".into())];
    assert_eq!(stack[0], "platform-monospace");
    assert_eq!(
        definitions.font_data[&stack[1]].tweak.monospace_cell_width,
        definitions.font_data[&stack[0]].tweak.monospace_cell_width
    );
    assert!(
        (width(&["monospace", "Pretendard"], "AA|")? - width(&["monospace", "Pretendard"], "한|")?)
            .abs()
            < 0.01
    );
    Ok(())
}

#[test]
fn one_line_uses_the_primary_for_latin_and_the_configured_fallback_for_hangul() -> Result<()> {
    let mut catalog = FontCatalog::default();
    let mut definitions = catalog.definitions_for_stack(&[
        "Geist Mono".into(),
        "Pretendard".into(),
        "Noto Sans CJK".into(),
    ])?;
    let stack = definitions.families[&appearance::editor_font(18.0).family].clone();
    assert!(
        definitions.font_data[&stack[1]].bytes() == definitions.font_data["Pretendard"].bytes(),
        "the configured fallback must precede the automatic bundled fonts"
    );
    for (role, key) in [("primary-only", &stack[0]), ("fallback-only", &stack[1])] {
        definitions
            .families
            .insert(FontFamily::Name(role.into()), vec![key.clone()]);
    }
    let context = Context::default();
    context.set_fonts(definitions);
    let mut output = context.run_ui(RawInput::default(), |ui| {
        ui.fonts_mut(|fonts| {
            let mixed =
                fonts.layout_no_wrap("A한".into(), appearance::editor_font(18.0), Color32::BLACK);
            let mut reference = LayoutJob::default();
            for (text, family) in [("A", "primary-only"), ("한", "fallback-only")] {
                reference.append(
                    text,
                    0.0,
                    TextFormat::simple(
                        FontId::new(18.0, FontFamily::Name(family.into())),
                        Color32::BLACK,
                    ),
                );
            }
            let expected = fonts.layout_job(reference);
            assert_eq!(mixed.rows[0].glyphs.len(), 2);
            assert_eq!(expected.rows[0].glyphs.len(), 2);
            // Match positions too: subpixel rasterization depends on the preceding glyph.
            let image = fonts.image();
            for (actual, expected) in mixed.rows[0].glyphs.iter().zip(&expected.rows[0].glyphs) {
                assert!(!actual.uv_rect.is_nothing());
                assert_eq!(actual.uv_rect.size, expected.uv_rect.size);
                assert_eq!(glyph_pixels(&image, actual), glyph_pixels(&image, expected));
            }
        });
    });
    output.textures_delta.clear();
    Ok(())
}

#[test]
fn a_saved_stack_survives_theme_changes_restart_and_rejected_writes() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let context = Context::default();
    app.note.body = "한글 💡 café".into();
    app.store.save(&app.note)?;
    let names = vec!["Noto Emoji".into(), "Geist".into(), "Pretendard".into()];
    app.set_editor_fonts(Some(&names), &context)?;
    let saved = app
        .store
        .setting("editorFont")?
        .expect("persisted preference");
    assert_eq!(serde_json::from_str::<Vec<String>>(&saved)?, names);
    app.set_theme("graphite", &context)?;
    assert_eq!(app.theme.typography.font_families(), names);
    let before = app.theme.clone();
    Connection::open(directory.path().join("notes.sqlite3"))?.execute_batch(
        "CREATE TRIGGER reject_fonts BEFORE INSERT ON app_state BEGIN SELECT RAISE(ABORT,'font fixture'); END;"
    )?;
    assert!(
        app.set_editor_fonts(Some(&["Geist".into(), "Noto Emoji".into()]), &context)
            .is_err()
    );
    assert_eq!(app.theme, before);
    assert_eq!(app.store.setting("editorFont")?, Some(saved));
    assert_eq!(app.note.body, "한글 💡 café");
    drop(app);
    let restored = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(restored.theme.typography.font_families(), names);
    assert_eq!(restored.note.body, "한글 💡 café");
    Ok(())
}

#[test]
fn theme_json_preserves_fallbacks_and_rejects_invalid_stacks() -> Result<()> {
    let mut theme = Theme::builtin("helium");
    assert!(theme.typography.font_fallbacks.is_empty());
    theme.typography.font_fallbacks = vec!["Pretendard".into(), "Noto Sans CJK".into()];
    assert_eq!(Theme::decode(&serde_json::to_vec(&theme)?)?, theme);
    theme.typography.font_fallbacks.push("pretendard".into());
    assert!(theme.validate().is_err());
    theme.typography.font_fallbacks = (0..8).map(|n| format!("Family {n}")).collect();
    assert!(theme.validate().is_err());
    Ok(())
}

#[test]
fn editor_font_changes_preserve_the_markdown_monospace_grid() -> Result<()> {
    let mut catalog = FontCatalog::default();
    let mut reference = None;
    for names in [
        vec!["monospace".into()],
        vec!["Geist Mono".into(), "Pretendard".into()],
    ] {
        let context = Context::default();
        context.set_fonts(catalog.definitions_for_stack(&names)?);
        let mut widths = Vec::new();
        let mut output = context.run_ui(RawInput::default(), |ui| {
            widths = ui.fonts_mut(|fonts| {
                ["AA|", "한|", "💡|"]
                    .map(|text| {
                        fonts
                            .layout_no_wrap(text.into(), FontId::monospace(18.0), Color32::BLACK)
                            .size()
                            .x
                    })
                    .to_vec()
            });
        });
        output.textures_delta.clear();
        for width in &widths[1..] {
            assert!((width - widths[0]).abs() < 0.01, "{names:?}: {widths:?}");
        }
        if let Some(reference) = &reference {
            assert_eq!(
                &widths, reference,
                "an editor preference must not change Markdown advances"
            );
        }
        reference = Some(widths);
    }
    Ok(())
}
