//! Preview the active glyph-fallback chain without changing a note or loading extra faces.
use super::QuickNoteApp;
use crate::appearance;
use eframe::egui::Frame;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::Ui;

impl QuickNoteApp {
    pub(super) fn font_preview(&self, ui: &mut Ui) {
        let text = self.locale.text();
        let palette = self.theme.palette();
        Frame::new()
            .fill(palette.toolbar)
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(&text.font_preview)
                        .small()
                        .color(palette.secondary),
                );
                ui.add(
                    Label::new(
                        RichText::new(&text.font_preview_sample)
                            .font(appearance::editor_font(self.theme.typography.font_size))
                            .color(palette.foreground),
                    )
                    .wrap(),
                );
            });
    }
}
