//! Theme-file tools and document actions retain their existing execution paths.
use super::QuickNoteApp;
use super::settings_section::SettingsSection;
use super::theme_files::ThemeAction;
use crate::i18n;
use crate::ui;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::Ui;

impl QuickNoteApp {
    pub(super) fn settings_actions(&mut self, ui: &mut Ui) {
        let palette = self.theme.palette();
        let mut operation = None;
        let mut command = None;
        ui.strong(self.locale.text().settings_themes.as_str());
        ui.horizontal_wrapped(|ui| {
            for (label, action) in [
                (&self.locale.text().import_theme, ThemeAction::ImportTheme),
                (&self.locale.text().export_theme, ThemeAction::ExportTheme),
                (
                    &self.locale.text().open_themes_folder,
                    ThemeAction::OpenThemesFolder,
                ),
                (&self.locale.text().reload_theme, ThemeAction::ReloadTheme),
            ] {
                let response = ui.button(label.as_str());
                ui::focus_ring(ui, &response, palette);
                if response.clicked() {
                    operation = Some(action);
                }
            }
        });
        ui.add_space(16.0);
        ui.strong(self.locale.text().notes.as_str());
        let title = crate::ui::note_title(&self.note.summary(), self.locale.text()).to_owned();
        ui.add(
            Label::new(
                RichText::new(i18n::format(
                    &self.locale.text().current_note,
                    &[("title", &title)],
                ))
                .color(palette.secondary),
            )
            .truncate(),
        );
        ui.horizontal_wrapped(|ui| {
            for (label, action) in [
                (&self.locale.text().export_note, ThemeAction::ExportNote),
                (&self.locale.text().find_note_action, ThemeAction::FindNote),
            ] {
                let response = ui.button(label.as_str());
                ui::focus_ring(ui, &response, palette);
                if response.clicked() {
                    operation = Some(action);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (label, action) in [
                (
                    &self.locale.text().recently_deleted,
                    crate::shortcut::Command::Trash,
                ),
                (
                    &self.locale.text().delete_note,
                    crate::shortcut::Command::Delete,
                ),
            ] {
                if ui.button(label.as_str()).clicked() {
                    command = Some(action);
                }
            }
        });
        ui.add_space(16.0);
        let response = ui.button(self.locale.text().keyboard_shortcuts.as_str());
        ui::focus_ring(ui, &response, palette);
        if response.clicked() {
            self.open_settings(SettingsSection::Shortcuts, ui.ctx());
        }
        ui.add_space(16.0);
        ui.weak(self.locale.text().utf8_notice.as_str());
        if let Some(command) = command {
            self.command(command, ui.ctx());
        }
        if let Some(operation) = operation {
            self.theme_action(operation, ui.ctx());
        }
    }
}
