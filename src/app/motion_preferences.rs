//! User preferences may reduce desktop motion further, but never override an OS reduction.
use super::QuickNoteApp;
use crate::appearance;
use crate::i18n;
use crate::storage::Result;
use crate::ui;
use crate::ui::motion;
use eframe::egui::Context;
use eframe::egui::Ui;

impl QuickNoteApp {
    #[must_use]
    pub fn reduced_motion(&self) -> bool {
        self.reduce_motion || self.system_reduce_motion
    }

    /// Persistence succeeds before visual behavior changes; notes are never rewritten.
    pub fn set_reduced_motion(&mut self, reduced: bool, context: &Context) -> Result<()> {
        self.store
            .set_setting("reduceMotion", if reduced { "true" } else { "false" })?;
        self.reduce_motion = reduced;
        motion::configure(context, self.reduced_motion());
        Ok(())
    }

    pub(super) fn configure_motion(&mut self, context: &Context) {
        self.system_reduce_motion = appearance::system_reduced_motion();
        motion::configure(context, self.reduced_motion());
    }

    pub(super) fn motion_option(&mut self, ui: &mut Ui) {
        let response = ui
            .add_enabled_ui(!self.system_reduce_motion, |ui| {
                ui::toggle_switch(
                    ui,
                    &self.locale.text().reduce_motion,
                    self.reduced_motion(),
                    self.theme.palette(),
                )
            })
            .inner;
        if response.clicked()
            && let Err(error) = self.set_reduced_motion(!self.reduce_motion, ui.ctx())
        {
            self.error = Some(
                i18n::message(|m| &m.error_motion_setting)
                    .cause("error", error)
                    .into(),
            );
        }
        response.on_hover_text(self.locale.text().motion_hint.as_str());
        if self.system_reduce_motion {
            ui.weak(self.locale.text().motion_system.as_str());
        }
    }
}
