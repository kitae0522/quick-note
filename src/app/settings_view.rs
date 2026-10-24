//! Settings occupies the document area, with persistent navigation and scrollable content.
use super::QuickNoteApp;
use super::settings_section::SettingsSection;
use super::shortcuts;
use crate::ui;
use crate::ui::safe_area;
use eframe::egui::Button;
use eframe::egui::CentralPanel;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;

impl QuickNoteApp {
    pub(super) fn settings_view(&mut self, ui: &mut Ui, opacity: f32) {
        let Some(section) = self.settings else {
            return;
        };
        let context = ui.ctx().clone();
        let palette = self.theme.palette();
        let mut back = false;
        self.focus_editor = false;
        Panel::top("settings-navigation")
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(palette.background)
                    .inner_margin(eframe::egui::Margin::symmetric(0, 16)),
            )
            .show(ui, |ui| {
                ui.multiply_opacity(opacity);
                settings_column(ui, |ui| {
                    ui.horizontal(|ui| {
                        let response = ui.add(
                            Button::new(self.locale.text().back_to_note.as_str())
                                .frame_when_inactive(false),
                        );
                        ui::focus_ring(ui, &response, palette);
                        back = response.clicked();
                        if section != SettingsSection::General {
                            let response = ui.button(self.locale.text().back_to_options.as_str());
                            if self.settings_focus_new
                                && section == SettingsSection::Shortcuts
                                && !self.close_blocked
                                && self.pending_delete.is_none()
                            {
                                response.request_focus();
                                self.settings_focus_new = false;
                            }
                            ui::focus_ring(ui, &response, palette);
                            if response.clicked() {
                                self.open_settings(SettingsSection::General, &context);
                            }
                        }
                    });
                    ui.add_space(12.0);
                    let title = match section {
                        SettingsSection::General => &self.locale.text().options,
                        SettingsSection::Fonts => &self.locale.text().editor_font,
                        SettingsSection::Shortcuts => &self.locale.text().keyboard_shortcuts,
                    };
                    ui.heading(
                        RichText::new(title)
                            .font(FontId::new(22.0, FontFamily::Name("medium".into()))),
                    );
                })
            });
        CentralPanel::default()
            .frame(Frame::new().fill(palette.background))
            .show(ui, |ui| {
                ui.multiply_opacity(opacity);
                settings_column(ui, |ui| {
                    ui.spacing_mut().item_spacing = eframe::egui::vec2(12.0, 8.0);
                    if section == SettingsSection::Fonts {
                        self.font_options(ui);
                    } else {
                        ScrollArea::vertical()
                            .id_salt(("settings-content", section))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if section == SettingsSection::Shortcuts {
                                    self.shortcut_controls(ui);
                                    shortcuts::show_help(
                                        ui,
                                        &self.shortcuts,
                                        self.locale.text(),
                                        self.markdown_enabled,
                                    );
                                } else {
                                    self.settings_general(ui);
                                }
                                ui.add_space(24.0);
                            });
                    }
                });
            });
        if back {
            self.close_settings(&context);
        }
    }
}

fn settings_column(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    let width = (ui.available_width() - 2.0 * f32::from(safe_area::EDGE_INSET)).clamp(0.0, 640.0);
    let offset = ((ui.available_width() - width) / 2.0).max(0.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(offset);
        ui.vertical(|ui| {
            ui.set_width(width);
            content(ui);
        });
    });
}
