//! Refresh the native status menu after changes, without idle polling.
use super::QuickNoteApp;
#[cfg(target_os = "macos")]
use crate::i18n;
use eframe::egui::Context;
use std::time::Duration;
use std::time::Instant;

impl QuickNoteApp {
    pub fn install_resident(&mut self, context: &Context) {
        self.install_menu(context);
        #[cfg(target_os = "macos")]
        {
            let result = self.store.recent_summaries().and_then(|notes| {
                if let Some(menu) = &mut self.native_menu {
                    menu.install_resident(self.locale.text(), notes)?;
                }
                Ok(())
            });
            if let Err(error) = result {
                self.error = Some(
                    i18n::message(|m| &m.error_menu)
                        .cause("error", error)
                        .into(),
                );
            }
        }
    }

    #[must_use]
    pub fn resident_installed(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.native_menu
                .as_ref()
                .is_some_and(crate::menu::MacMenu::resident_installed)
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    pub(super) fn update_resident(&mut self, context: &Context) {
        if !self.resident_installed() {
            self.capture.recent_dirty = false;
            self.capture.recent_due = None;
            self.capture.saved_until = None;
            return;
        }
        let now = Instant::now();
        if self
            .capture
            .saved_until
            .is_some_and(|deadline| now >= deadline)
        {
            self.capture.saved_until = None;
            #[cfg(target_os = "macos")]
            if let Some(menu) = &self.native_menu {
                menu.capture_feedback(None);
            }
        }
        if std::mem::take(&mut self.capture.recent_dirty) {
            self.capture.recent_due = Some(now + Duration::from_millis(300));
            context.request_repaint_after(Duration::from_millis(300));
        }
        if self
            .capture
            .recent_due
            .is_some_and(|deadline| now >= deadline)
        {
            self.capture.recent_due = None;
            #[cfg(target_os = "macos")]
            if let Some(menu) = &mut self.native_menu {
                let result = self
                    .store
                    .recent_summaries()
                    .and_then(|notes| menu.refresh_recent(self.locale.text(), notes));
                if let Err(error) = result {
                    self.error = Some(
                        i18n::message(|m| &m.error_menu)
                            .cause("error", error)
                            .into(),
                    );
                }
            }
        }
        for deadline in [self.capture.recent_due, self.capture.saved_until]
            .into_iter()
            .flatten()
        {
            context.request_repaint_after(deadline.saturating_duration_since(now));
        }
    }

    #[cfg(all(target_os = "macos", feature = "diagnostics"))]
    pub fn resident_menu(&self) -> Option<objc2::rc::Retained<objc2_app_kit::NSMenu>> {
        self.native_menu.as_ref()?.resident_menu()
    }
}
