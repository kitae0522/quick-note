//! Automatic optical sizing follows reading size while explicit font choices retain priority.
#![cfg(target_os = "macos")]
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::RawInput;
use eframe::egui::epaint::text::VariationCoords;
use eframe::egui::text::LayoutJob;
use quick_note::appearance;
use quick_note::appearance::FontCatalog;
use quick_note::storage::Result;

#[test]
fn variable_fonts_use_the_reading_size_instead_of_display_spacing() -> Result<()> {
    let mut catalog = FontCatalog::default();
    let definitions = catalog.definitions_for_stack(&["SF Pro".into(), "Pretendard".into()])?;
    let context = Context::default();
    context.set_fonts(definitions);
    let mut output = context.run_ui(RawInput::default(), |ui| {
        ui.fonts_mut(|fonts| {
            for size in [11.0, 14.0, 18.0, 32.0] {
                for text in [
                    "AVATAR To. Writing feels immediate.",
                    "Quick Note에서 아이디어를 기록해요.",
                ] {
                    let natural = fonts.layout_no_wrap(
                        text.into(),
                        appearance::editor_font(size),
                        Color32::BLACK,
                    );
                    let mut job = LayoutJob::simple(
                        text.into(),
                        appearance::editor_font(size),
                        Color32::BLACK,
                        f32::INFINITY,
                    );
                    job.sections[0].format.coords = VariationCoords::new([(b"opsz", size)]);
                    let optical = fonts.layout_job(job);
                    assert!(
                        (natural.size().x - optical.size().x).abs() < 0.01,
                        "{size}pt {text}: default {} != optical {}",
                        natural.size().x,
                        optical.size().x
                    );
                }
            }
        });
    });
    output.textures_delta.clear();
    Ok(())
}

#[test]
fn explicit_font_and_run_optical_sizes_are_preserved() -> Result<()> {
    let mut catalog = FontCatalog::default();
    let mut definitions = catalog.definitions("SF Pro")?;
    let data = definitions
        .font_data
        .get_mut("local-editor")
        .expect("resolved SF Pro");
    std::sync::Arc::make_mut(data)
        .tweak
        .coords
        .push(b"opsz", 28.0);
    let context = Context::default();
    context.set_fonts(definitions);
    let mut output = context.run_ui(RawInput::default(), |ui| {
        ui.fonts_mut(|fonts| {
            let text = "AVATAR To. Writing feels immediate.";
            let default =
                fonts.layout_no_wrap(text.into(), appearance::editor_font(14.0), Color32::BLACK);
            let mut job = LayoutJob::simple(
                text.into(),
                appearance::editor_font(14.0),
                Color32::BLACK,
                f32::INFINITY,
            );
            job.sections[0].format.coords = VariationCoords::new([(b"opsz", 28.0)]);
            assert_eq!(default.size(), fonts.layout_job(job.clone()).size());
            job.sections[0].format.coords = VariationCoords::new([(b"opsz", 17.0)]);
            assert!((default.size().x - fonts.layout_job(job).size().x).abs() > 1.0);
        });
    });
    output.textures_delta.clear();
    Ok(())
}
