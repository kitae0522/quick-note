use super::Mode;
use super::QuickNoteApp;
use crate::i18n;
use crate::storage::Result;
use crate::ui;
use eframe::egui::Ui;
use egui_commonmark::AlertBundle;

impl QuickNoteApp {
    #[must_use]
    pub fn markdown_enabled(&self) -> bool {
        self.markdown_enabled
    }

    /// Saves the choice before changing rendering state. Disabling drops all viewer caches.
    pub fn set_markdown_enabled(&mut self, enabled: bool) -> Result<()> {
        self.store
            .set_setting("markdownEnabled", if enabled { "true" } else { "false" })?;
        self.markdown_enabled = enabled;
        if !enabled {
            self.markdown_cache = None;
            if self.mode == Mode::Preview {
                self.mode = Mode::Edit;
                self.previous_focus = None;
                self.restore_cursor = true;
                self.focus_editor = !self.settings_open() && !self.search_open;
            }
        }
        Ok(())
    }

    pub(super) fn markdown_option(&mut self, ui: &mut Ui) {
        let response = ui::toggle_switch(
            ui,
            &self.locale.text().markdown,
            self.markdown_enabled,
            self.theme.palette(),
        );
        if response.clicked()
            && let Err(error) = self.set_markdown_enabled(!self.markdown_enabled)
        {
            self.error = Some(
                i18n::message(|m| &m.error_markdown_setting)
                    .cause("error", error)
                    .into(),
            );
        }
        response.on_hover_text(self.locale.text().markdown_hint.as_str());
    }

    pub(super) fn markdown_alerts(&self) -> AlertBundle {
        let mut alerts = AlertBundle::gfm().into_alerts();
        let text = self.locale.text();
        for alert in &mut alerts {
            alert.identifier_rendered = match alert.identifier.as_str() {
                "NOTE" => &text.alert_note,
                "TIP" => &text.alert_tip,
                "IMPORTANT" => &text.alert_important,
                "WARNING" => &text.alert_warning,
                "CAUTION" => &text.alert_caution,
                _ => continue,
            }
            .clone();
        }
        AlertBundle::from_alerts(alerts)
    }
}
