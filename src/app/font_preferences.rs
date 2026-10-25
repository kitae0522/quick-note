//! Persist the editor font independently of colors and load saved local fonts off the UI path.
use super::QuickNoteApp;
use super::files;
use crate::appearance;
use crate::appearance::Theme;
use crate::i18n;
use crate::storage::Result;
use crate::storage::Store;
use eframe::egui::Context;

impl QuickNoteApp {
    /// Selects a bundled/installed family, or restores the active theme's own family.
    /// Failed font lookup or persistence leaves the current font and notes unchanged.
    pub fn set_editor_font(&mut self, name: Option<&str>, context: &Context) -> Result<()> {
        let names = name.map(|name| vec![name.to_owned()]);
        self.set_editor_fonts(names.as_deref(), context)
    }

    /// Applies an ordered font stack only after both resolution and persistence succeed.
    pub fn set_editor_fonts(&mut self, names: Option<&[String]>, context: &Context) -> Result<()> {
        let mut theme = self.theme.clone();
        let preference = match names {
            Some(names) => {
                let names: Vec<_> = names.iter().map(|name| name.trim().to_owned()).collect();
                let Some((first, rest)) = names.split_first() else {
                    return Err(i18n::message(|m| &m.error_font_stack).into());
                };
                theme.typography.font_family = first.clone();
                theme.typography.font_fallbacks = rest.to_vec();
                if names.len() == 1 {
                    names[0].clone()
                } else {
                    serde_json::to_string(&names)?
                }
            }
            None => {
                let typography = files::read_theme(&self.directory, &self.theme_id)?.typography;
                theme.typography.font_family = typography.font_family;
                theme.typography.font_fallbacks = typography.font_fallbacks;
                String::new()
            }
        };
        theme.validate()?;
        let fonts = self
            .fonts
            .definitions_for_stack(&theme.typography.font_families())?;
        self.store.set_setting("editorFont", &preference)?;
        context.set_fonts(fonts);
        self.theme = theme;
        self.markdown_cache = None;
        self.font_pending = false;
        self.theme_error = None;
        Ok(())
    }

    pub(super) fn apply_font_preference(&self, theme: &mut Theme) -> Result<()> {
        apply_saved(&self.store, theme)
    }

    pub(super) fn configure_editor_font(&mut self, context: &Context) {
        self.font_pending = true;
        if let Err(error) = self.fonts.request(context) {
            self.font_pending = false;
            self.theme_error = Some(error);
        }
        if !self.install_ready_fonts(context) {
            appearance::install_fonts(context);
        }
    }

    pub(super) fn poll_fonts(&mut self, context: &Context) {
        if let Err(error) = self.fonts.poll() {
            self.font_pending = false;
            self.theme_error = Some(
                i18n::message(|m| &m.error_font_list)
                    .cause("error", error)
                    .into(),
            );
        }
        self.install_ready_fonts(context);
    }

    fn install_ready_fonts(&mut self, context: &Context) -> bool {
        if self.font_pending && self.fonts.names().is_some() {
            self.font_pending = false;
            match self
                .fonts
                .definitions_for_stack(&self.theme.typography.font_families())
            {
                Ok(fonts) => {
                    context.set_fonts(fonts);
                    self.markdown_cache = None;
                    return true;
                }
                Err(error) => self.theme_error = Some(error),
            }
        }
        false
    }
}

pub(super) fn apply_saved(store: &Store, theme: &mut Theme) -> Result<()> {
    if let Some(value) = store
        .setting("editorFont")?
        .filter(|value| !value.is_empty())
    {
        let names = serde_json::from_str::<Vec<String>>(&value).unwrap_or_else(|_| vec![value]);
        let Some((first, rest)) = names.split_first() else {
            return Err(i18n::message(|m| &m.error_font_stack).into());
        };
        let mut candidate = theme.clone();
        candidate.typography.font_family = first.clone();
        candidate.typography.font_fallbacks = rest.to_vec();
        candidate.validate()?;
        *theme = candidate;
    }
    Ok(())
}
