//! Persist the editor font independently of colors and load saved local fonts off the UI path.
use super::QuickNoteApp;
use super::files;
use crate::appearance;
use crate::appearance::Theme;
use crate::i18n;
use crate::storage::Result;
use eframe::egui::Context;

impl QuickNoteApp {
    /// Selects a bundled/installed family, or restores the active theme's own family.
    /// Failed font lookup or persistence leaves the current font and notes unchanged.
    pub fn set_editor_font(&mut self, name: Option<&str>, context: &Context) -> Result<()> {
        let mut theme = self.theme.clone();
        theme.typography.font_family = match name {
            Some(name) => name.trim().into(),
            None => {
                files::read_theme(&self.directory, &self.theme_id)?
                    .typography
                    .font_family
            }
        };
        theme.validate()?;
        let fonts = self.fonts.definitions(&theme.typography.font_family)?;
        self.store
            .set_setting("editorFont", name.map_or("", str::trim))?;
        context.set_fonts(fonts);
        self.theme = theme;
        self.markdown_cache = None;
        self.font_pending = false;
        self.theme_error = None;
        Ok(())
    }

    pub(super) fn apply_font_preference(&self, theme: &mut Theme) -> Result<()> {
        if let Some(font) = self
            .store
            .setting("editorFont")?
            .filter(|value| !value.is_empty())
        {
            theme.typography.font_family = font;
        }
        Ok(())
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
            match self.fonts.definitions(&self.theme.typography.font_family) {
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
