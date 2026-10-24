use super::QuickNoteApp;
use crate::i18n::Language;
use crate::i18n::Localizer;
use crate::storage::Result;
use eframe::egui::Context;
use eframe::egui::ViewportCommand;

impl QuickNoteApp {
    /// Validates translations before saving the choice; a failed write keeps the current language.
    pub fn set_language(&mut self, language: Language, context: &Context) -> Result<()> {
        let locale = Localizer::load(language, &self.directory)?;
        self.store.set_setting("language", language.code())?;
        context.send_viewport_cmd(ViewportCommand::Title(locale.text().app_name.clone()));
        self.locale = locale;
        self.invalidate_menu();
        self.markdown_cache = None;
        self.search_query = None;
        self.apply_language(context);
        Ok(())
    }
    pub(super) fn apply_language(&self, context: &Context) {
        let text = self.locale.text();
        egui_commonmark_backend::labels::CodeBlockLabels {
            read_only: text.code_block.clone(),
            copy: text.copy.clone(),
            copied: text.copied.clone(),
            copy_accessible: text.copy_code.clone(),
            copied_accessible: text.copy_code_again.clone(),
        }
        .install(context);
    }
}
