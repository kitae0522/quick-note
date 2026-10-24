//! Verify actual glyph and cursor positions rather than just font-family names.
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::RawInput;
use eframe::egui::ViewportId;
use eframe::egui::text::CCursor;
use eframe::egui::text::LayoutJob;
use quick_note::appearance;
use quick_note::appearance::FontCatalog;
use quick_note::storage::Result;

#[test]
fn default_editor_columns_align_across_scripts_sizes_and_display_scales() -> Result<()> {
    let mut catalog = FontCatalog::default();
    let native = if cfg!(target_os = "windows") {
        "Consolas"
    } else {
        "Menlo"
    };
    for family in ["monospace", native] {
        for scale in [1.0, 1.25, 2.0] {
            let context = Context::default();
            context.set_fonts(catalog.definitions(family)?);
            let mut input = RawInput::default();
            input
                .viewports
                .get_mut(&ViewportId::ROOT)
                .expect("default raw input includes its root viewport")
                .native_pixels_per_point = Some(scale);
            let mut output = context.run_ui(input, |ui| {
                ui.fonts_mut(|fonts| {
                    for size in [11.0, 14.0, 18.0, 32.0] {
                        let font = appearance::editor_font(size);
                        let reference =
                            fonts.layout_no_wrap("iiii|".into(), font.clone(), Color32::BLACK);
                        let expected = reference.rows[0].glyphs[4].pos.x;
                        for prefix in [
                            "WWWW",
                            "0000",
                            "    ",
                            "AVAV",
                            "ffil",
                            "->=>",
                            "{.} ",
                            "한글",
                            "가나",
                            "中文",
                            "日本",
                            "💡💡",
                            "🇰🇷🇺🇸",
                            "👨‍👩‍👧‍👦💡",
                            "e\u{301}e\u{301}e\u{301}e\u{301}",
                        ] {
                            let galley = fonts.layout_no_wrap(
                                format!("{prefix}|"),
                                font.clone(),
                                Color32::BLACK,
                            );
                            let glyph = galley.rows[0].glyphs.last().expect("delimiter exists");
                            assert!(
                                (glyph.pos.x - expected).abs() < 0.01,
                                "{family}, {size}pt, {scale}x, {prefix:?}: {} != {expected}",
                                glyph.pos.x,
                            );
                            let caret =
                                galley.pos_from_cursor(CCursor::new(prefix.chars().count()));
                            assert!((caret.min.x - expected).abs() < 0.01);
                            if prefix.is_ascii() {
                                for index in 0..4 {
                                    let cursor = CCursor::new(index);
                                    let expected = reference.pos_from_cursor(cursor).min.x;
                                    assert!(
                                        (galley.pos_from_cursor(cursor).min.x - expected).abs()
                                            < 0.01
                                    );
                                }
                            }
                        }
                    }
                });
            });
            output.textures_delta.clear();
        }
    }
    Ok(())
}

#[test]
fn grid_matching_retains_cjk_and_emoji_outline_sizes() -> Result<()> {
    let context = Context::default();
    let mut catalog = FontCatalog::default();
    context.set_fonts(catalog.definitions("monospace")?);
    let mut output = context.run_ui(RawInput::default(), |ui| {
        ui.fonts_mut(|fonts| {
            for (name, text) in [("Noto Sans CJK", "한"), ("Noto Emoji", "💡")] {
                let natural = fonts.layout_no_wrap(
                    text.into(),
                    FontId::new(14.0, FontFamily::Name(name.into())),
                    Color32::BLACK,
                );
                let grid = fonts.layout_no_wrap(
                    text.into(),
                    appearance::editor_font(14.0),
                    Color32::BLACK,
                );
                assert_eq!(
                    natural.rows[0].glyphs[0].uv_rect.size,
                    grid.rows[0].glyphs[0].uv_rect.size
                );
                assert!(
                    (natural.rows[0].glyphs[0].advance_width
                        - grid.rows[0].glyphs[0].advance_width)
                        .abs()
                        > 0.1
                );
            }
        });
    });
    output.textures_delta.clear();
    Ok(())
}

#[test]
fn wrapping_uses_the_same_cells_without_losing_unicode_cursor_positions() -> Result<()> {
    let context = Context::default();
    let mut catalog = FontCatalog::default();
    context.set_fonts(catalog.definitions("monospace")?);
    let mut output = context.run_ui(RawInput::default(), |ui| {
        ui.fonts_mut(|fonts| {
            let font = appearance::editor_font(14.0);
            let cell = fonts.glyph_width(&font, '0');
            for text in ["WWWWWWWW", "한글한글", "💡💡💡💡", "👨‍👩‍👧‍👦💡💡💡"]
            {
                let mut job =
                    LayoutJob::simple(text.into(), font.clone(), Color32::BLACK, 6.0 * cell);
                job.wrap.break_anywhere = true;
                let galley = fonts.layout_job(job);
                assert_eq!(galley.rows.len(), 2, "{text}");
                assert!((galley.rows[0].size.x - 6.0 * cell).abs() <= 1.0, "{text}");
                assert_eq!(galley.end().index.0, text.chars().count());
            }
        });
    });
    output.textures_delta.clear();
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn platform_defaults_use_the_actual_native_monospace_and_ui_faces() -> Result<()> {
    let mut catalog = FontCatalog::default();
    let defaults = catalog.definitions("monospace")?;
    let name = if cfg!(target_os = "windows") {
        "Consolas"
    } else {
        "Menlo"
    };
    let explicit = catalog.definitions(name)?;
    assert_eq!(
        defaults.font_data["platform-monospace"].bytes(),
        explicit.font_data["local-editor"].bytes()
    );
    assert_eq!(
        defaults.font_data["platform-monospace"].index,
        explicit.font_data["local-editor"].index
    );
    if cfg!(target_os = "macos") {
        let ui = catalog.definitions("SF Pro")?;
        assert_eq!(defaults.families[&FontFamily::Proportional][0], "native-ui");
        assert_eq!(
            defaults.font_data["native-ui"],
            ui.font_data["local-editor"]
        );
        assert_ne!(
            defaults.font_data["native-ui"],
            defaults.font_data["native-strong"]
        );
    }
    Ok(())
}
