//! Logical line numbers and the focused caret's gutter background share the text layout.
use crate::appearance::Palette;
use eframe::egui;
use eframe::egui::Align2;
use eframe::egui::FontId;
use eframe::egui::Rect;
use eframe::egui::Ui;
use eframe::egui::text_edit::TextEditOutput;

pub(super) fn paint(ui: &Ui, output: &TextEditOutput, current_line: usize, palette: Palette) {
    let mut number = 1;
    let mut first_fragment = true;
    for row in &output.galley.rows {
        let top = output.galley_pos.y + row.pos.y;
        let active = number == current_line && output.response.has_focus();
        if active {
            let rect = Rect::from_min_max(
                egui::pos2(output.response.rect.left(), top),
                egui::pos2(output.galley_pos.x - 6.0, top + row.size.y),
            );
            if ui.is_rect_visible(rect) {
                ui.painter().rect_filled(rect, 0, palette.current_line);
            }
        }
        if first_fragment {
            let position = egui::pos2(output.galley_pos.x - 12.0, top + row.size.y / 2.0);
            if ui.clip_rect().contains(position) {
                ui.painter().text(
                    position,
                    Align2::RIGHT_CENTER,
                    number.to_string(),
                    FontId::monospace(12.0),
                    if active {
                        palette.foreground
                    } else {
                        palette.secondary
                    },
                );
            }
        }
        first_fragment = row.ends_with_newline;
        if row.ends_with_newline {
            number += 1;
        }
    }
}
