//! Compact chrome leaves the editor available while keeping every action discoverable.
use super::QuickNoteApp;
use crate::shortcut::Command;
use crate::ui;
use crate::ui::header::HeaderOutput;
use eframe::egui;
use eframe::egui::Align;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::egui::ViewportCommand;

impl QuickNoteApp {
    pub(super) fn capture_options(&mut self, ui: &mut Ui) {
        ui.add_space(16.0);
        ui.strong(self.locale.text().quick_capture.as_str());
        ui.weak(crate::i18n::format(
            &self.locale.text().capture_hint,
            &[("shortcut", &self.shortcut_text(Command::Capture, ui.ctx()))],
        ));
        for (label, command) in [
            (&self.locale.text().capture.new_note, Command::NewCapture),
            (
                &self.locale.text().capture.clipboard,
                Command::CaptureClipboard,
            ),
        ] {
            ui.horizontal_wrapped(|ui| {
                ui.label(label);
                ui.weak(self.shortcut_text(command, ui.ctx()));
            });
        }
        ui.horizontal_wrapped(|ui| {
            let mut focus = self.capture.focused;
            if ui
                .checkbox(&mut focus, self.locale.text().focus_mode.as_str())
                .changed()
            {
                self.toggle_focus_mode(ui.ctx());
            }
            let mut pinned = self.capture.pinned;
            if ui
                .checkbox(&mut pinned, self.locale.text().pin_window.as_str())
                .changed()
            {
                self.toggle_pin(ui.ctx());
            }
        });
    }

    pub(super) fn capture_header(&mut self, ui: &mut Ui) -> HeaderOutput {
        let mut selected = None;
        let mut expand = false;
        let mut search_rect = egui::Rect::NOTHING;
        let palette = self.theme.palette();
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            if cfg!(target_os = "macos") {
                ui.add_space(76.0);
            }
            let title = self.note.summary();
            let title = ui::note_title(&title, self.locale.text());
            let response = ui.add_sized(
                [
                    (ui.available_width() - 3.0 * (32.0 + ui.spacing().item_spacing.x)).max(20.0),
                    ui.available_height(),
                ],
                Label::new(title).truncate().sense(Sense::drag()),
            );
            if response.drag_started() {
                ui.ctx().stop_dragging();
                ui.input_mut(|input| input.pointer = Default::default());
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }
            let search = ui::controls::icon_button(
                ui,
                ui::icons::SEARCH,
                &self.locale.text().find_note,
                palette,
            );
            search_rect = search.rect;
            if self.header.focus_search {
                search.request_focus();
                self.header.focus_search = false;
            }
            if search.clicked() {
                selected = Some(Command::Search);
            }
            if ui::controls::icon_button(ui, ui::icons::PLUS, &self.locale.text().new_note, palette)
                .clicked()
            {
                selected = Some(if self.capture.compact {
                    Command::NewCapture
                } else {
                    Command::New
                });
            }
            let more = ui::controls::icon_button(
                ui,
                ui::icons::MORE,
                &self.locale.text().capture_menu,
                palette,
            );
            eframe::egui::Popup::menu(&more).show(|ui| {
                if ui.button(self.locale.text().expand_note.as_str()).clicked() {
                    expand = true;
                    ui.close();
                }
                for (label, command) in [
                    (&self.locale.text().pin_window, Command::PinWindow),
                    (&self.locale.text().focus_mode, Command::FocusMode),
                    (&self.locale.text().recently_deleted, Command::Trash),
                    (&self.locale.text().delete_note, Command::Delete),
                    (&self.locale.text().options, Command::Options),
                ] {
                    if ui.button(label.as_str()).clicked() {
                        selected = Some(command);
                        ui.close();
                    }
                }
            });
        });
        if expand {
            self.expand_capture(ui.ctx());
        }
        if let Some(command) = selected {
            self.command(command, ui.ctx());
        }
        HeaderOutput {
            action: None,
            search_rect,
            search_changed: false,
            search_gained_focus: false,
        }
    }
}
