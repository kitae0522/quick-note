//! Match fallback advances to the editor grid while retaining their original outlines.
use crate::i18n;
use crate::storage::Result;
use eframe::egui::FontData;
use eframe::egui::FontDefinitions;
use skrifa::FontRef;
use skrifa::instance::Size;
use std::sync::Arc;

fn advance_em(data: &FontData, name: &str, character: char) -> Result<f32> {
    use skrifa::MetadataProvider as _;
    let font = FontRef::from_index(data.bytes(), data.index)?;
    let location = font.axes().location(data.tweak.coords.as_ref());
    let invalid = || i18n::message(|m| &m.error_font_metrics).arg("font", name);
    let glyph = font.charmap().map(character).ok_or_else(invalid)?;
    font.glyph_metrics(Size::new(1.0), &location)
        .advance_width(glyph)
        .map(|width| width * data.tweak.scale)
        .filter(|width| width.is_finite() && *width > 0.0)
        .ok_or_else(|| invalid().into())
}

pub(super) fn monospace(fonts: &mut FontDefinitions, primary: &str) -> Result<Vec<String>> {
    let cell = advance_em(&fonts.font_data[primary], primary, '0')?;
    let mut primary_data = (*fonts.font_data[primary]).clone();
    primary_data.tweak.monospace_cell_width = Some(cell);
    fonts
        .font_data
        .insert(primary.into(), Arc::new(primary_data));
    let mut family = vec![primary.into()];
    for (name, suffix) in [
        ("Noto Emoji", "emoji"),
        ("Noto Sans CJK", "cjk"),
        ("Pretendard", "korean"),
    ] {
        let mut data = (*fonts.font_data[name]).clone();
        data.tweak.monospace_cell_width = Some(cell);
        // Each primary owns its fallback metrics, including the separate Markdown grid.
        let key = format!("{primary}-{suffix}");
        fonts.font_data.insert(key.clone(), Arc::new(data));
        family.push(key);
    }
    Ok(family)
}
