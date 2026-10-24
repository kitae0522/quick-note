//! Embedded faces and startup fallbacks; native UI and monospace faces load asynchronously.
use super::font_stack;
use eframe::egui::Context;
use eframe::egui::FontData;
use eframe::egui::FontDefinitions;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use std::sync::Arc;

pub fn install_fonts(context: &Context) {
    context.set_fonts(bundled_fonts());
}

#[must_use]
pub fn editor_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("editor".into()))
}

/// Present platform presets with their preferred family rather than an internal theme alias.
#[must_use]
pub fn font_display_name(name: &str) -> &str {
    match name {
        "monospace" if cfg!(target_os = "macos") => "Menlo",
        "monospace" if cfg!(target_os = "windows") => "Consolas",
        _ => name,
    }
}

#[must_use]
pub fn is_bundled_font(name: &str) -> bool {
    matches!(
        name,
        "system"
            | "monospace"
            | "Pretendard"
            | "Noto Sans CJK"
            | "Geist"
            | "Geist Mono"
            | "Geist Medium"
            | "Geist Bold"
            | "Noto Emoji"
    )
}

pub(super) fn bundled_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in [
        (
            "Pretendard",
            include_bytes!("../../assets/fonts/Pretendard-Regular.ttf").as_slice(),
        ),
        (
            "Pretendard Medium",
            include_bytes!("../../assets/fonts/Pretendard-Medium.otf").as_slice(),
        ),
        (
            "Pretendard Bold",
            include_bytes!("../../assets/fonts/Pretendard-Bold.otf").as_slice(),
        ),
        (
            "Noto Sans CJK",
            include_bytes!("../../assets/fonts/NotoSansCJKkr-Regular.otf").as_slice(),
        ),
        (
            "Noto Emoji",
            include_bytes!("../../assets/fonts/NotoEmoji.ttf").as_slice(),
        ),
        (
            "Geist",
            include_bytes!("../../assets/fonts/Geist-Regular.ttf").as_slice(),
        ),
        (
            "Geist Medium",
            include_bytes!("../../assets/fonts/Geist-Medium.ttf").as_slice(),
        ),
        (
            "Geist Bold",
            include_bytes!("../../assets/fonts/Geist-Bold.ttf").as_slice(),
        ),
        (
            "Geist Mono",
            include_bytes!("../../assets/fonts/GeistMono-Regular.ttf").as_slice(),
        ),
        (
            "Phosphor",
            include_bytes!("../../assets/fonts/Phosphor.ttf").as_slice(),
        ),
    ] {
        fonts
            .font_data
            .insert(name.into(), Arc::new(FontData::from_static(bytes)));
        fonts
            .families
            .insert(FontFamily::Name(name.into()), vec![name.into()]);
    }
    let mut ui = vec![
        "Pretendard".into(),
        "Noto Sans CJK".into(),
        "Noto Emoji".into(),
    ];
    ui.extend(fonts.families[&FontFamily::Proportional].iter().cloned());
    fonts.families.insert(FontFamily::Proportional, ui.clone());
    let mut medium = ui.clone();
    medium.insert(0, "Pretendard Medium".into());
    fonts
        .families
        .insert(FontFamily::Name("medium".into()), medium);
    let mut strong = ui;
    strong.insert(0, "Pretendard Bold".into());
    fonts
        .families
        .insert(FontFamily::Name("strong".into()), strong);
    fonts
        .families
        .insert(FontFamily::Name("icons".into()), vec!["Phosphor".into()]);
    let mono = font_stack::monospace(&mut fonts, "Hack")
        .expect("bundled fonts contain the glyphs validated by spacing tests");
    fonts.families.insert(FontFamily::Monospace, mono.clone());
    fonts
        .families
        .insert(FontFamily::Name("editor".into()), mono);
    fonts
}
