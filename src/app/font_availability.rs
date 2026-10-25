//! Describe the faces the renderer can use, including portable but uninstalled presets.
use super::QuickNoteApp;
use crate::appearance;

impl QuickNoteApp {
    pub(super) fn font_available(&self, name: &str) -> bool {
        appearance::is_bundled_font(name)
            || (name == "SF Pro" && cfg!(target_os = "macos"))
            || self
                .fonts
                .names()
                .is_some_and(|fonts| fonts.iter().any(|font| font.eq_ignore_ascii_case(name)))
    }

    pub(super) fn font_source_label(&self, name: &str) -> &str {
        let text = self.locale.text();
        if matches!(name, "system" | "monospace") {
            &text.font_system
        } else if appearance::is_bundled_font(name) {
            &text.font_bundled
        } else if self.font_available(name) {
            &text.font_installed
        } else if self.fonts.names().is_none() {
            &text.loading_fonts
        } else {
            &text.font_unavailable
        }
    }
}
