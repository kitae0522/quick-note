//! Settings transitions preserve document state and let the editor finish the input batch.
use super::Mode;
use super::QuickNoteApp;
use super::settings_section::SettingsSection;
use eframe::egui::Context;
use eframe::egui::Id;

impl QuickNoteApp {
    #[must_use]
    pub fn settings_open(&self) -> bool {
        self.settings.is_some() || self.settings_request.is_some()
    }

    pub(super) fn open_settings(&mut self, section: SettingsSection, context: &Context) {
        if self.settings.is_some() {
            self.settings = Some(section);
        } else {
            self.settings_request = Some(section);
        }
        self.search_open = false;
        self.settings_focus_new = true;
        self.previous_focus = None;
        context.request_repaint();
    }

    pub(super) fn finish_settings_navigation(&mut self, context: &Context) {
        if let Some(section) = self.settings_request.take() {
            self.settings = Some(section);
            self.focus_editor = false;
            self.settings_focus_new = true;
            context.memory_mut(|memory| {
                memory.surrender_focus(Id::unique(("note-editor", &self.note.id)));
            });
            context.request_repaint();
        }
    }

    pub(super) fn close_settings(&mut self, context: &Context) {
        self.settings = None;
        self.settings_request = None;
        self.settings_focus_new = false;
        self.search_open = false;
        self.restore_cursor = true;
        self.focus_editor = self.mode == Mode::Edit;
        if self.focus_editor {
            context.memory_mut(|memory| {
                memory.request_focus(Id::unique(("note-editor", &self.note.id)))
            });
        } else if let Some(id) = context.memory(|memory| memory.focused()) {
            context.memory_mut(|memory| memory.surrender_focus(id));
        }
        context.request_repaint();
    }

    pub(super) fn restore_focus(&mut self, context: &Context) {
        if let Some(id) = self.previous_focus {
            context.memory_mut(|memory| memory.request_focus(id));
        }
        let editor = Id::unique(("note-editor", &self.note.id));
        self.focus_editor =
            self.mode == Mode::Edit && self.previous_focus.is_none_or(|id| id == editor);
    }
}
