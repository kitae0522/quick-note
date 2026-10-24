//! Theme selection, reload and the options window.
use super::QuickNoteApp;
use super::files;
use crate::i18n;
use crate::storage::Result;
use eframe::egui::Context;

impl QuickNoteApp {
    pub fn set_theme(&mut self, id: &str, context: &Context) -> Result<()> {
        let mut theme = files::read_theme(&self.directory, id)?;
        self.apply_font_preference(&mut theme)?;
        let fonts = self.fonts.definitions(&theme.typography.font_family)?;
        self.store.set_setting("theme", id)?;
        context.set_fonts(fonts);
        theme.apply(context);
        self.theme = theme;
        self.markdown_cache = None;
        self.theme_id = id.into();
        self.theme_error = None;
        self.font_pending = false;
        Ok(())
    }

    pub(super) fn reload_theme(&mut self, context: &Context) {
        let result: Result<crate::appearance::Theme> = (|| {
            let mut theme = files::read_theme(&self.directory, &self.theme_id)?;
            self.apply_font_preference(&mut theme)?;
            if theme.typography.font_family != self.theme.typography.font_family
                || self.theme_error.is_some()
            {
                context.set_fonts(self.fonts.definitions(&theme.typography.font_family)?);
                self.font_pending = false;
            }
            Ok(theme)
        })();
        match result {
            Ok(theme) => {
                theme.apply(context);
                self.theme = theme;
                self.markdown_cache = None;
                self.theme_error = None;
            }
            Err(error) => {
                self.theme_error = Some(
                    i18n::message(|m| &m.error_theme_reload)
                        .cause("error", error)
                        .into(),
                );
            }
        }
    }
}
