//! Destructive and failed-save decisions default to preserving the draft.
use super::QuickNoteApp;
use crate::i18n;
use crate::ui;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::Modal;
use eframe::egui::RichText;
use eframe::egui::Stroke;
use eframe::egui::TextStyle;
use eframe::egui::ViewportCommand;
use eframe::egui::text::LayoutJob;

impl QuickNoteApp {
    pub(super) fn delete_dialog(&mut self, context: &Context, opacity: f32) {
        let Some(id) = self.pending_delete.as_deref() else {
            return;
        };
        let title = if id == self.note.id {
            crate::ui::note_title(&self.note.summary(), self.locale.text()).to_owned()
        } else {
            self.tabs.iter().find(|tab| tab.id == id).map_or_else(
                || self.locale.text().note_untitled.clone(),
                |tab| crate::ui::note_title(tab, self.locale.text()).to_owned(),
            )
        };
        let palette = self.theme.palette();
        let mut confirm = false;
        let mut cancel = false;
        let result = decision_modal(context, "delete-note-confirmation").show(context, |ui| {
            ui.multiply_opacity(opacity);
            ui.set_width((context.content_rect().width() - 72.0).clamp(200.0, 320.0));
            ui.spacing_mut().item_spacing = eframe::egui::vec2(12.0, 0.0);
            ui.heading(RichText::new(self.locale.text().delete_title.as_str()).size(18.0));
            ui.add_space(12.0);
            let mut body = LayoutJob::simple(
                i18n::format(&self.locale.text().delete_body, &[("title", &title)]),
                TextStyle::Body.resolve(ui.style()),
                palette.foreground,
                ui.available_width(),
            );
            // Long titles stay readable through the label's full-text tooltip and accessibility text.
            body.wrap.max_rows = 3;
            ui.add(Label::new(body).wrap());
            ui.add_space(6.0);
            ui.label(
                RichText::new(self.locale.text().delete_warning.as_str()).color(palette.secondary),
            );
            ui.add_space(20.0);
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 224.0).max(0.0));
                let response = ui.add_sized(
                    [100.0, 32.0],
                    Button::new(self.locale.text().cancel.as_str()),
                );
                if self.modal_new {
                    response.request_focus();
                    self.modal_new = false;
                }
                ui::focus_ring(ui, &response, palette);
                cancel = response.clicked();
                let response = ui.add_sized(
                    [112.0, 32.0],
                    Button::new(
                        RichText::new(self.locale.text().delete_note.as_str())
                            .color(palette.on_danger),
                    )
                    .fill(palette.danger)
                    .stroke(Stroke::NONE),
                );
                ui::focus_ring(
                    ui,
                    &response,
                    crate::appearance::Palette {
                        accent: palette.on_danger,
                        ..palette
                    },
                );
                confirm = response.clicked();
            });
        });
        if confirm {
            if let Err(error) = self.confirm_delete() {
                self.error = Some(
                    i18n::message(|m| &m.error_delete)
                        .cause("error", error)
                        .into(),
                );
            }
        } else if cancel || result.should_close() {
            self.cancel_delete();
            self.restore_focus(context);
        }
    }

    pub(super) fn close_dialog(&mut self, context: &Context, opacity: f32) {
        let response = decision_modal(context, "unsaved-before-close").show(context, |ui| {
            ui.multiply_opacity(opacity);
            ui.set_width((context.content_rect().width() - 72.0).clamp(200.0, 340.0));
            ui.spacing_mut().item_spacing = eframe::egui::vec2(12.0, 0.0);
            ui.heading(RichText::new(self.locale.text().close_unsaved_title.as_str()).size(18.0));
            ui.add_space(12.0);
            ui.label(self.locale.text().close_unsaved_body.as_str());
            ui.add_space(20.0);
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.horizontal_wrapped(|ui| {
                let keep = ui.add_sized(
                    [100.0, 32.0],
                    Button::new(self.locale.text().keep_editing.as_str()),
                );
                if self.modal_new {
                    keep.request_focus();
                    self.modal_new = false;
                }
                if keep.clicked() {
                    self.close_blocked = false;
                    self.close_settings(context);
                }
                if ui
                    .add_sized(
                        [80.0, 32.0],
                        Button::new(self.locale.text().retry_save.as_str()),
                    )
                    .clicked()
                    && self.persist()
                {
                    self.close_blocked = false;
                    context.send_viewport_cmd(ViewportCommand::Close);
                }
                if ui
                    .add_sized(
                        [120.0, 32.0],
                        Button::new(self.locale.text().export_text.as_str()),
                    )
                    .clicked()
                    && self.export_note()
                {
                    self.close_blocked = false;
                    self.dirty = false;
                    context.send_viewport_cmd(ViewportCommand::Close);
                }
            });
        });
        if response.should_close() {
            self.close_blocked = false;
            self.focus_editor = true;
        }
    }
}

fn decision_modal(context: &Context, id: &str) -> Modal {
    Modal::new(Id::unique(id))
        .backdrop_color(Color32::from_black_alpha(
            if context.style_of(context.theme()).visuals.dark_mode {
                104
            } else {
                72
            },
        ))
        .frame(
            Frame::popup(&context.style_of(context.theme()))
                .inner_margin(20)
                .corner_radius(16),
        )
}
