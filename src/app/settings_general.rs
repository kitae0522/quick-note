//! Language, appearance and editor preferences grouped on the settings page.
use super::QuickNoteApp;
use crate::appearance;
use crate::appearance::Theme;
use crate::i18n;
use crate::i18n::Language;
use crate::ui;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::Ui;
use std::fs;

impl QuickNoteApp {
    pub(super) fn settings_general(&mut self, ui: &mut Ui) {
        let palette = self.theme.palette();
        let mut language_choice = None;
        let mut theme_choice = None;
        ui.strong(self.locale.text().language.as_str());
        ui.horizontal_wrapped(|ui| {
            for (language, label) in [
                (Language::English, self.locale.text().language_en.as_str()),
                (Language::Korean, self.locale.text().language_ko.as_str()),
            ] {
                let response =
                    ui::controls::choice_button(ui, label, self.locale.language == language, 108.0);
                if self.settings_focus_new && !self.close_blocked && self.pending_delete.is_none() {
                    response.request_focus();
                    response.scroll_to_me(None);
                    self.settings_focus_new = false;
                }
                ui::focus_ring(ui, &response, palette);
                if response.clicked() {
                    language_choice = Some(language);
                }
            }
        });
        ui.add_space(16.0);
        ui.strong(self.locale.text().appearance.as_str());
        let mut themes = vec![
            ("helium".to_owned(), self.locale.text().theme_light.clone()),
            ("graphite".to_owned(), self.locale.text().theme_dark.clone()),
        ];
        if let Ok(files) = fs::read_dir(self.directory.join("Themes")) {
            for file in files.flatten().filter(|file| {
                file.path()
                    .extension()
                    .is_some_and(|extension| extension == "json")
            }) {
                let id = file.file_name().to_string_lossy().into_owned();
                let title = Theme::read(&file.path()).map_or_else(
                    |_| i18n::format(&self.locale.text().check_theme, &[("id", &id)]),
                    |theme| theme.name,
                );
                themes.push((id, title));
            }
        }
        ui.horizontal_wrapped(|ui| {
            for (id, title) in themes {
                let response = ui::controls::choice_button(ui, &title, self.theme_id == id, 200.0);
                ui::focus_ring(ui, &response, palette);
                if response.clicked() {
                    theme_choice = Some(id);
                }
            }
        });
        ui.add_space(16.0);
        ui.strong(self.locale.text().settings_editor.as_str());
        ui.horizontal_wrapped(|ui| {
            let response = ui.button(self.locale.text().choose_font.as_str());
            ui::focus_ring(ui, &response, palette);
            if response.clicked() {
                self.open_font_options(ui.ctx());
            }
            ui.add(
                Label::new(
                    RichText::new(appearance::font_stack_display_name(
                        &self.theme.typography.font_families(),
                    ))
                    .color(palette.secondary),
                )
                .truncate(),
            );
        });
        self.markdown_option(ui);
        self.motion_option(ui);
        self.capture_options(ui);
        #[cfg(target_os = "macos")]
        self.login_option(ui);
        ui.add_space(16.0);
        self.settings_actions(ui);
        if let Some(language) = language_choice
            && let Err(error) = self.set_language(language, ui.ctx())
        {
            self.error = Some(
                i18n::message(|m| &m.error_language)
                    .cause("error", error)
                    .into(),
            );
        }
        if let Some(id) = theme_choice
            && let Err(error) = self.set_theme(&id, ui.ctx())
        {
            self.theme_error = Some(error);
        }
    }
}
