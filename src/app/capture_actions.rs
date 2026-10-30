//! Reuse a live editor for quick capture; hide only after its text is safely committed.
use super::Mode;
use super::QuickNoteApp;
use crate::i18n;
use crate::shortcut::Command;
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::ViewportCommand;
use eframe::egui::WindowLevel;

impl QuickNoteApp {
    /// Register the configured global shortcut only for a real native application.
    pub fn install_capture(&mut self, context: &Context) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            let keys = [
                Command::Capture,
                Command::NewCapture,
                Command::CaptureClipboard,
            ]
            .into_iter()
            .flat_map(|command| {
                self.shortcuts
                    .keys(command)
                    .map(move |keys| (command, keys))
            })
            .collect();
            let result: crate::storage::Result<()> = (|| {
                if self.capture.native.is_none() {
                    self.capture.native = Some(crate::capture::GlobalCapture::new(context)?);
                }
                self.capture
                    .native
                    .as_mut()
                    .expect("native hotkey manager was initialized above")
                    .update(keys)
            })();
            if let Err(error) = result {
                self.shortcut_error = Some(
                    i18n::message(|m| &m.error_capture)
                        .cause("error", error)
                        .into(),
                );
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let _ = context;
    }

    pub(super) fn receive_capture(&mut self, context: &Context) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(native) = &self.capture.native {
            for action in native.receive() {
                self.capture.pending.push_back(action);
            }
        }
        if self.pending_delete.is_some() || self.close_blocked {
            self.capture.pending.clear();
            return;
        }
        if !self.composing
            && !context.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, Event::Ime(_)))
            })
            && let Some((command, was_active)) = self.capture.pending.pop_front()
        {
            if command == Command::Capture {
                self.toggle_capture(context, was_active);
            } else {
                self.command(command, context);
            }
        }
    }

    pub(super) fn toggle_capture(&mut self, context: &Context, was_active: bool) {
        if self.composing
            || context.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, Event::Ime(_)))
            })
        {
            self.capture
                .pending
                .push_back((Command::Capture, was_active));
            context.request_repaint();
            return;
        }
        if self.pending_delete.is_some() || self.close_blocked {
            return;
        }
        if self.capture.compact && was_active {
            self.hide_capture(context);
            return;
        }
        if !self.capture.compact {
            self.capture.normal_rect = context.input(|input| input.viewport().inner_rect);
            self.capture.normal_position =
                context.input(|input| input.viewport().outer_rect.map(|rect| rect.min));
            context.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(520.0, 340.0)));
        }
        self.capture.compact = true;
        self.settings = None;
        self.settings_request = None;
        self.search_open = false;
        self.document_find.close();
        self.mode = Mode::Edit;
        self.focus_editor = true;
        self.restore_cursor = true;
        Self::show_window(context);
    }

    pub(super) fn hide_capture(&mut self, context: &Context) {
        self.capture.hide_pending = true;
        // Escape clears egui focus before layout; accept this frame's final text before hiding.
        self.focus_editor = self.mode == Mode::Edit;
        context.request_repaint();
    }

    pub(super) fn finish_capture(&mut self, context: &Context) {
        if self.composing || !std::mem::take(&mut self.capture.hide_pending) {
            return;
        }
        if self.dirty && !self.persist() {
            return;
        }
        self.discard_capture_draft();
        #[cfg(target_os = "macos")]
        if let Some(main) = objc2::MainThreadMarker::new() {
            objc2_app_kit::NSApplication::sharedApplication(main).hide(None);
            // AppKit keeps the window available to both the global shortcut and the Dock.
            return;
        }
        context.send_viewport_cmd(ViewportCommand::Visible(false));
    }

    pub(super) fn expand_capture(&mut self, context: &Context) {
        if self.capture.compact {
            let size = self
                .capture
                .normal_rect
                .map_or(egui::vec2(920.0, 600.0), |rect| rect.size());
            context.send_viewport_cmd(ViewportCommand::InnerSize(size));
            self.capture.normal_rect = None;
            if let Some(position) = self.capture.normal_position.take() {
                context.send_viewport_cmd(ViewportCommand::OuterPosition(position));
            }
        }
        self.capture.compact = false;
        self.capture.focused = false;
        self.focus_editor = true;
        self.header.reveal_active = true;
    }

    pub(super) fn toggle_focus_mode(&mut self, context: &Context) {
        self.capture.focused = !self.capture.focused;
        self.focus_editor = true;
        self.search_open = false;
        context.request_repaint();
    }

    pub(super) fn toggle_pin(&mut self, context: &Context) {
        self.capture.pinned = !self.capture.pinned;
        context.send_viewport_cmd(ViewportCommand::WindowLevel(if self.capture.pinned {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        }));
    }

    pub(super) fn quiet_editor(&self) -> bool {
        self.capture.compact || self.capture.focused
    }

    #[must_use]
    pub fn capture_registered(&self) -> bool {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            self.capture
                .native
                .as_ref()
                .is_some_and(crate::capture::GlobalCapture::registered)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            false
        }
    }

    #[must_use]
    pub fn capture_open(&self) -> bool {
        self.capture.compact
    }
}
