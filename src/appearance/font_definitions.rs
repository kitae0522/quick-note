//! Resolve ordered editor families before automatic fallbacks, preserving the primary grid.
use super::font_stack;
use super::fonts;
use super::system_fonts::SystemFonts;
use crate::i18n;
use crate::storage::Result;
use eframe::egui::FontDefinitions;
use eframe::egui::FontFamily;
use fontique::FontWeight;
use std::sync::Arc;

pub(super) fn definitions(native: &mut SystemFonts, names: &[String]) -> Result<FontDefinitions> {
    if names.is_empty() || names.len() > 8 {
        return Err(i18n::message(|m| &m.error_font_stack).into());
    }
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
    definitions.families.insert(FontFamily::Monospace, mono);
    let mut family = Vec::new();
    let mut grid = None;
    let mut automatic = Vec::new();
    let mut unavailable = None;
    for (index, name) in names.iter().enumerate() {
        let key = match name.as_str() {
            "system" => definitions.families[&FontFamily::Proportional][0].clone(),
            "monospace" => "platform-monospace".into(),
            _ if fonts::is_bundled_font(name) => name.clone(),
            _ => match native.load(name) {
                Ok(data) => {
                    let key = if family.is_empty() {
                        "local-editor".into()
                    } else {
                        format!("local-editor-{index}")
                    };
                    definitions.font_data.insert(key.clone(), Arc::new(data));
                    key
                }
                Err(error) => {
                    unavailable.get_or_insert(error);
                    continue;
                }
            },
        };
        let key = if family.is_empty() {
            if name == "monospace"
                || name.eq_ignore_ascii_case("Menlo")
                || name.eq_ignore_ascii_case("Consolas")
                || name == "Geist Mono"
            {
                automatic = font_stack::monospace(&mut definitions, &key)?;
                grid = definitions.font_data[&key].tweak.monospace_cell_width;
            }
            key
        } else if let Some(cell) = grid {
            let editor_key = format!("editor-selected-{index}");
            let mut data = (*definitions.font_data[&key]).clone();
            data.tweak.monospace_cell_width = Some(cell);
            definitions
                .font_data
                .insert(editor_key.clone(), Arc::new(data));
            editor_key
        } else {
            key
        };
        if !family.contains(&key) {
            family.push(key);
        }
    }
    if family.is_empty() {
        return Err(unavailable.expect("every family in a nonempty unresolved stack failed lookup"));
    }
    for fallback in automatic
        .iter()
        .chain(&definitions.families[&FontFamily::Proportional])
    {
        if !family.contains(fallback) {
            family.push(fallback.clone());
        }
    }
    definitions
        .families
        .insert(FontFamily::Name("editor".into()), family);
    // Font definitions retain all selected faces; release unused source-cache entries.
    native.cache.prune(1, false);
    Ok(definitions)
}
