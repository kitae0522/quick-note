//! Replacing a binding set is transactional; failed reloads retain every current binding.
use super::QuickNoteApp;
use crate::i18n;
use crate::shortcut::Command;
use crate::shortcut::Platform;
use crate::shortcut::Shortcuts;
use crate::ui;
use eframe::egui::Context;
use eframe::egui::Ui;

impl QuickNoteApp {
    pub fn reload_shortcuts(&mut self) {
        match Shortcuts::load(&self.directory, Platform::current()) {
            Ok(shortcuts) => {
                self.shortcuts = shortcuts;
                self.invalidate_menu();
                self.shortcut_error = None;
            }
            Err(error) => {
                self.shortcut_error = Some(
                    i18n::message(|m| &m.error_shortcut_reload)
                        .cause("error", error)
                        .into(),
                )
            }
        }
    }

    pub(super) fn open_shortcuts_folder(&mut self) {
        if let Err(error) = open::that(self.directory.join("shortcut")) {
            self.shortcut_error = Some(
                i18n::message(|m| &m.error_shortcut_folder)
                    .cause("error", error)
                    .into(),
            );
        }
    }

    pub(super) fn shortcut_text(&self, command: Command, context: &Context) -> String {
        let keys = self.shortcuts.key_text(command, context);
        if keys.is_empty() {
            self.locale.text().shortcut_unassigned.clone()
        } else {
            keys
        }
    }

    pub(super) fn shortcut_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            let open = ui.button(self.locale.text().open_shortcuts_folder.as_str());
            ui::focus_ring(ui, &open, self.theme.palette());
            if open.clicked() {
                self.open_shortcuts_folder();
            }
            let reload = ui.button(self.locale.text().reload_shortcuts.as_str());
            ui::focus_ring(ui, &reload, self.theme.palette());
            if reload.clicked() {
                self.reload_shortcuts();
            }
        });
        ui.weak(self.locale.text().shortcuts_hint.as_str());
        ui.add_space(16.0);
    }
}
