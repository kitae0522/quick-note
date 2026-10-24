//! Assemble native UI weights and a selected editor face without changing font metrics.
use super::font_stack;
use super::fonts;
use super::system_fonts::SystemFonts;
use crate::storage::Result;
use eframe::egui::FontDefinitions;
use eframe::egui::FontFamily;
use fontique::FontWeight;
use std::sync::Arc;

pub(super) fn definitions(native: &mut SystemFonts, name: &str) -> Result<FontDefinitions> {
    let mut definitions = fonts::bundled_fonts();
    for (key, family, weight) in [
        ("native-ui", FontFamily::Proportional, FontWeight::NORMAL),
        (
            "native-medium",
            FontFamily::Name("medium".into()),
            FontWeight::MEDIUM,
        ),
        (
            "native-strong",
            FontFamily::Name("strong".into()),
            FontWeight::BOLD,
        ),
    ] {
        if let Some(data) = native.ui(weight) {
            definitions.font_data.insert(key.into(), Arc::new(data));
            definitions
                .families
                .get_mut(&family)
                .expect("bundled definitions include UI roles")
                .insert(0, key.into());
        }
    }
    let platform = native.monospace()?;
    definitions
        .font_data
        .insert("platform-monospace".into(), Arc::new(platform));
    let mono = font_stack::monospace(&mut definitions, "platform-monospace")?;
    definitions
        .families
        .insert(FontFamily::Monospace, mono.clone());
    let mut family = match name {
        "system" => definitions.families[&FontFamily::Proportional].clone(),
        "monospace" => mono,
        _ if fonts::is_bundled_font(name) => vec![name.into()],
        _ => {
            definitions
                .font_data
                .insert("local-editor".into(), Arc::new(native.load(name)?));
            if name.eq_ignore_ascii_case("Menlo") || name.eq_ignore_ascii_case("Consolas") {
                font_stack::monospace(&mut definitions, "local-editor")?
            } else {
                vec!["local-editor".into()]
            }
        }
    };
    for fallback in &definitions.families[&FontFamily::Proportional] {
        if !family.contains(fallback) {
            family.push(fallback.clone());
        }
    }
    definitions
        .families
        .insert(FontFamily::Name("editor".into()), family);
    // Keep active defaults and the selected face, releasing previous selections.
    native.cache.prune(1, false);
    Ok(definitions)
}
