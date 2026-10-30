//! Clipboard reads happen only on an explicit capture command.
use super::QuickNoteApp;
use crate::i18n;
use eframe::egui::Context;
use std::time::Duration;
use std::time::Instant;

impl QuickNoteApp {
    pub(super) fn capture_clipboard(&mut self, context: &Context) {
        let result = arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.get_text())
            .map_err(|error| {
                i18n::Error::from(
                    i18n::message(|m| &m.capture.error_clipboard).cause("error", error),
                )
            })
            .and_then(|text| self.capture_text(text));
        match result {
            Ok(title) => {
                let message = i18n::format(
                    &self.locale.text().capture.clipboard_saved,
                    &[("title", &title)],
                );
                #[cfg(target_os = "macos")]
                if let Some(menu) = &self.native_menu {
                    menu.capture_feedback(Some(&message));
                }
                #[cfg(not(target_os = "macos"))]
                let _ = message;
                self.capture.saved_until = Some(Instant::now() + Duration::from_millis(1600));
                context.request_repaint_after(Duration::from_millis(1600));
            }
            Err(error) => {
                self.error.get_or_insert(error);
                self.toggle_capture(context, false);
            }
        }
    }
}
