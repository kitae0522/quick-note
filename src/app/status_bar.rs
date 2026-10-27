//! Editor mode, save state and cursor location.
use super::Mode;
use super::QuickNoteApp;
use crate::shortcut::Command;
use crate::ui;
use crate::ui::safe_area;
use eframe::egui;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Layout;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

impl QuickNoteApp {
    pub(super) fn status_bar(&mut self, ui: &mut Ui) {
        let context = ui.ctx().clone();
        let palette = self.theme.palette();
        Panel::bottom("status")
            .frame(
                Frame::new()
                    .fill(palette.background)
                    .inner_margin(Margin::symmetric(safe_area::EDGE_INSET, 4)),
            )
            .exact_size(32.0)
            .show(ui, |ui| {
                ui.spacing_mut().interact_size.y = 24.0;
                ui.spacing_mut().button_padding = egui::vec2(8.0, 3.0);
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for mode in [Mode::Edit, Mode::Preview] {
                        if mode == Mode::Preview && !self.markdown_enabled {
                            continue;
                        }
                        let label = if mode == Mode::Edit {
                            self.locale.text().edit.as_str()
                        } else {
                            self.locale.text().preview.as_str()
                        };
                        let response = ui.add(
                            Button::new(RichText::new(label).size(12.0))
                                .selected(self.mode == mode)
                                .frame(false)
                                .min_size(egui::vec2(40.0, 24.0)),
                        );
                        response.widget_info(|| {
                            WidgetInfo::selected(Role::Button, true, self.mode == mode, label)
                        });
                        ui::focus_ring(ui, &response, palette);
                        if self.mode == mode {
                            ui.painter().hline(
                                response.rect.left() + 8.0..=response.rect.right() - 8.0,
                                response.rect.bottom() - 1.0,
                                Stroke::new(2.0, palette.accent),
                            );
                        }
                        if response.clicked() {
                            if self.mode != mode {
                                self.command(Command::Preview, &context);
                            }
                            self.focus_editor = mode == Mode::Edit;
                        }
                        if self.markdown_enabled {
                            response.on_hover_text(crate::i18n::format(
                                &self.locale.text().shortcut_hint,
                                &[
                                    ("action", self.locale.text().shortcut_preview.as_str()),
                                    ("shortcut", &self.shortcut_text(Command::Preview, &context)),
                                ],
                            ));
                        }
                    }
                    if self.dirty {
                        if ui
                            .button(
                                RichText::new(if context.content_rect().width() < 480.0 {
                                    self.locale.text().retry_save.as_str()
                                } else {
                                    self.locale.text().not_saved_retry.as_str()
                                })
                                .size(12.0)
                                .color(palette.foreground),
                            )
                            .clicked()
                        {
                            let _saved = self.persist();
                        }
                    } else {
                        let response = ui.label(
                            RichText::new(self.locale.text().saved.as_str())
                                .size(12.0)
                                .color(palette.secondary),
                        );
                        response.widget_info(|| {
                            WidgetInfo::labeled(
                                Role::Status,
                                true,
                                self.locale.text().saved.as_str(),
                            )
                        });
                        context.accesskit_node_builder(response.id, |node| {
                            node.set_live(egui::accesskit::Live::Polite)
                        });
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(self.locale.text().encoding.as_str())
                                .size(12.0)
                                .color(palette.secondary),
                        );
                        ui.label(
                            RichText::new(crate::i18n::format(
                                &self.locale.text().cursor_location,
                                &[
                                    ("line", &self.location.line.to_string()),
                                    ("column", &self.location.column.to_string()),
                                ],
                            ))
                            .font(FontId::monospace(12.0))
                            .color(palette.secondary),
                        );
                    });
                });
            });
    }
}
