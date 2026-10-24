//! Native theme-file commands shared by keyboard and options.
use super::QuickNoteApp;
use super::files;
use crate::appearance::Theme;
use crate::i18n;
use crate::storage::Result;
use eframe::egui::Context;

#[derive(Clone, Copy)]
pub(super) enum ThemeAction {
    ImportTheme,
    ExportTheme,
    OpenThemesFolder,
    ReloadTheme,
    ExportNote,
    FindNote,
}

impl QuickNoteApp {
    pub(super) fn theme_action(&mut self, operation: ThemeAction, context: &Context) {
        let result: Result<()> = (|| {
            match operation {
                ThemeAction::ImportTheme => {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title(&self.locale.text().import_theme)
                        .add_filter(self.locale.text().theme_filter.as_str(), &["json"])
                        .pick_file()
                    {
                        let theme = Theme::read(&path)?;
                        let id = format!("{}.json", uuid::Uuid::new_v4());
                        files::atomic_write(
                            &self.directory.join("Themes").join(&id),
                            &serde_json::to_vec_pretty(&theme)?,
                        )?;
                        self.set_theme(&id, context)?;
                    }
                }
                ThemeAction::ExportTheme => {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title(&self.locale.text().export_theme)
                        .add_filter(self.locale.text().json_theme_filter.as_str(), &["json"])
                        .set_file_name(format!(
                            "{}.json",
                            files::safe_filename(
                                &self.theme.name,
                                &self.locale.text().note_untitled
                            )
                        ))
                        .save_file()
                    {
                        files::atomic_write(&path, &serde_json::to_vec_pretty(&self.theme)?)?;
                    }
                }
                ThemeAction::OpenThemesFolder => {
                    open::that(self.directory.join("Themes"))?;
                }
                ThemeAction::ReloadTheme => self.reload_theme(context),
                ThemeAction::ExportNote => {
                    self.export_note();
                }
                ThemeAction::FindNote => self.request_search(context),
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.error = Some(
                i18n::message(|m| &m.error_theme_action)
                    .cause("error", error)
                    .into(),
            );
        }
    }
}
