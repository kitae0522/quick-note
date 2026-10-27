//! Find uses one row when its controls fit, retaining a separate row in narrow windows.
use super::QuickNoteApp;
use crate::shortcut::Command;
use crate::ui;
use eframe::egui::Align;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Layout;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::Role;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

fn text_width(ui: &Ui, text: &str, size: f32) -> f32 {
    ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(
                text.into(),
                FontId::proportional(size),
                ui.visuals().text_color(),
            )
            .size()
            .x
    })
}

impl QuickNoteApp {
    pub(super) fn find_bar(&mut self, ui: &mut Ui) {
        if !self.document_find.open {
            return;
        }
        self.document_find.refresh(&self.note);
        let palette = self.theme.palette();
        let context = ui.ctx().clone();
        let status = self.document_find.status(self.locale.text());
        let mut changed = false;
        let mut next = None;
        let mut replace = None;
        let mut close = false;
        Panel::bottom("document-find")
            .frame(
                Frame::new()
                    .fill(palette.toolbar)
                    .inner_margin(Margin::symmetric(16, 6)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                let gap = ui.spacing().item_spacing.x;
                let text = self.locale.text();
                let case_width = text_width(ui, &text.find_match_case, 14.0) + 24.0;
                let mut options_width = text_width(ui, &text.replace_one, 14.0)
                    + 24.0
                    + 2.0 * ui::controls::CONTROL_HEIGHT
                    + 2.0 * gap;
                if !status.is_empty() {
                    options_width += text_width(ui, &status, 14.0) + gap;
                }
                let controls_width = case_width + ui::controls::CONTROL_HEIGHT + 2.0 * gap;
                // Preserve a useful query width; translation and result counts set the breakpoint.
                let wide = ui.available_width() >= 160.0 + controls_width + options_width + gap;
                let query_width = (ui.available_width()
                    - controls_width
                    - if wide { options_width + gap } else { 0.0 })
                .max(32.0);
                ui.horizontal(|ui| {
                    changed |= self.find_query_input(ui, query_width);
                    let response = ui.checkbox(
                        &mut self.document_find.match_case,
                        &self.locale.text().find_match_case,
                    );
                    changed |= response.changed();
                    ui::focus_ring(ui, &response, palette);
                    if wide {
                        next = ui
                            .allocate_ui_with_layout(
                                eframe::egui::vec2(options_width, ui::controls::CONTROL_HEIGHT),
                                Layout::left_to_right(Align::Center),
                                |ui| self.find_options(ui, &status),
                            )
                            .inner;
                    }
                    close = ui::icon_button(
                        ui,
                        ui::icons::CLOSE,
                        &self.locale.text().find_close,
                        palette,
                    )
                    .clicked();
                });
                if self.document_find.show_replace {
                    ui.horizontal(|ui| {
                        let text = self.locale.text();
                        let reserve = [&text.replace_one, &text.replace_all]
                            .iter()
                            .map(|label| {
                                text_width(ui, label, 14.0) + 2.0 * ui.spacing().button_padding.x
                            })
                            .sum::<f32>()
                            + 2.0 * gap;
                        self.find_replacement_input(ui, (ui.available_width() - reserve).max(32.0));
                        ui.add_enabled_ui(
                            !self.composing && !self.document_find.matches.is_empty(),
                            |ui| {
                                for (label, all) in [
                                    (&self.locale.text().replace_one, false),
                                    (&self.locale.text().replace_all, true),
                                ] {
                                    let response = ui.button(label);
                                    ui::focus_ring(ui, &response, palette);
                                    if response.clicked() {
                                        replace = Some(all);
                                    }
                                }
                            },
                        );
                    });
                }
                if !wide {
                    ui.horizontal(|ui| next = self.find_options(ui, &status));
                }
            });
        if changed {
            self.document_find.invalidate();
            self.document_find.refresh(&self.note);
            self.select_find_match();
            context.request_repaint();
        }
        if let Some(command) = next {
            self.find_command(command, &context);
        }
        if let Some(all) = replace {
            self.replace_find(all, &context);
        }
        if close {
            self.close_find();
        }
    }

    fn find_options(&mut self, ui: &mut Ui, status: &str) -> Option<Command> {
        let text = self.locale.text();
        let palette = self.theme.palette();
        let response = ui.checkbox(&mut self.document_find.show_replace, &text.replace_one);
        response.widget_info(|| {
            WidgetInfo::selected(
                Role::CheckBox,
                true,
                self.document_find.show_replace,
                &text.find_toggle_replace,
            )
        });
        ui::focus_ring(ui, &response, palette);
        if !status.is_empty() {
            let response = ui.weak(status);
            response.widget_info(|| WidgetInfo::labeled(Role::Status, true, status));
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_live(eframe::egui::accesskit::Live::Polite)
            });
        }
        let mut next = None;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_enabled_ui(
                !self.document_find.matches.is_empty() && !self.composing,
                |ui| {
                    for (glyph, label, command) in [
                        (ui::icons::DOWN, &text.find_next, Command::FindNext),
                        (ui::icons::UP, &text.find_previous, Command::FindPrevious),
                    ] {
                        if ui::icon_button(ui, glyph, label, palette).clicked() {
                            next = Some(command);
                        }
                    }
                },
            );
        });
        next
    }
}
