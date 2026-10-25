//! Validated, shareable UTF-8 design tokens.
use super::Palette;
use crate::i18n;
use crate::storage::Result;
use eframe::egui::Color32;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub schema_version: u32,
    pub name: String,
    pub appearance: String,
    pub colors: Colors,
    pub typography: Typography,
    pub layout: Layout,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Colors {
    pub background: String,
    pub toolbar: String,
    pub foreground: String,
    pub secondary: String,
    pub accent: String,
    pub divider: String,
    pub selection: String,
    #[serde(
        rename = "currentLine",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub current_line: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Typography {
    pub font_family: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub font_fallbacks: Vec<String>,
    pub font_size: f32,
    pub line_spacing: f32,
}

impl Typography {
    #[must_use]
    pub fn font_families(&self) -> Vec<String> {
        std::iter::once(self.font_family.clone())
            .chain(self.font_fallbacks.iter().cloned())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub editor_padding: f32,
    pub toolbar_height: f32,
}

impl Theme {
    #[must_use]
    pub fn builtin(id: &str) -> Self {
        let source = if id == "graphite" {
            include_str!("../../assets/themes/graphite.json")
        } else {
            include_str!("../../assets/themes/helium.json")
        };
        Self::decode(source.as_bytes()).expect("built-in themes are validated in tests")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 64 * 1024 {
            return Err(i18n::message(|m| &m.error_theme_size).into());
        }
        let theme: Self = serde_json::from_slice(bytes)?;
        theme.validate()?;
        Ok(theme)
    }

    pub fn read(path: &Path) -> Result<Self> {
        use std::io::Read;
        let mut bytes = Vec::new();
        File::open(path)?
            .take(64 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        Self::decode(&bytes)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(i18n::message(|m| &m.error_theme_version).into());
        }
        if self.name.trim().is_empty() || self.name.chars().count() > 60 {
            return Err(i18n::message(|m| &m.error_theme_name).into());
        }
        if !matches!(self.appearance.as_str(), "light" | "dark") {
            return Err(i18n::message(|m| &m.error_theme_appearance).into());
        }
        for value in [
            &self.colors.background,
            &self.colors.toolbar,
            &self.colors.foreground,
            &self.colors.secondary,
            &self.colors.accent,
            &self.colors.divider,
            &self.colors.selection,
        ]
        .into_iter()
        .chain(self.colors.current_line.iter())
        {
            if hex(value).is_none() {
                return Err(i18n::message(|m| &m.error_theme_color)
                    .arg("value", value)
                    .into());
            }
        }
        let families = self.typography.font_families();
        if families.len() > 8
            || families.iter().enumerate().any(|(index, name)| {
                families[..index]
                    .iter()
                    .any(|other| name.eq_ignore_ascii_case(other))
            })
        {
            return Err(i18n::message(|m| &m.error_font_stack).into());
        }
        for name in families {
            if name.trim().is_empty() || name.chars().count() > 100 {
                return Err(i18n::message(|m| &m.error_theme_font_name).into());
            }
        }
        if !(11.0..=32.0).contains(&self.typography.font_size)
            || !(0.0..=24.0).contains(&self.typography.line_spacing)
            || !(12.0..=64.0).contains(&self.layout.editor_padding)
            || !(32.0..=56.0).contains(&self.layout.toolbar_height)
        {
            return Err(i18n::message(|m| &m.error_theme_layout).into());
        }
        Ok(())
    }

    /// Converts validated color tokens to renderer values.
    ///
    /// # Panics
    /// Panics if callers mutate public colors without validating them before use.
    #[must_use]
    pub fn palette(&self) -> Palette {
        let color =
            |value: &str| hex(value).expect("Theme::validate checks colors before applying");
        Palette {
            background: color(&self.colors.background),
            toolbar: color(&self.colors.toolbar),
            foreground: color(&self.colors.foreground),
            secondary: color(&self.colors.secondary),
            accent: color(&self.colors.accent),
            divider: color(&self.colors.divider),
            selection: color(&self.colors.selection),
            current_line: self.colors.current_line.as_deref().map_or_else(
                || {
                    color(&self.colors.background)
                        .lerp_to_gamma(color(&self.colors.foreground), 0.1)
                },
                color,
            ),
            danger: Color32::from_rgb(178, 45, 56),
            on_danger: Color32::WHITE,
        }
    }
}

fn hex(value: &str) -> Option<Color32> {
    if value.len() != 7
        || !value.starts_with('#')
        || !value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    let value = u32::from_str_radix(&value[1..], 16).ok()?;
    Some(Color32::from_rgb(
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ))
}
