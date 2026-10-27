//! Paint matches without splitting shaping runs or changing the editor's glyph spacing.
use crate::appearance::Palette;
use crate::model::TextMatch;
use eframe::egui::Align;
use eframe::egui::Rect;
use eframe::egui::Shape;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::Ui;
use eframe::egui::layers::ShapeIdx;
use eframe::egui::text::CCursor;
use eframe::egui::text_edit::TextEditOutput;

pub(crate) struct FindHighlights<'a> {
    pub matches: &'a [TextMatch],
    pub active: usize,
    pub reveal: bool,
}

pub(super) fn paint(
    ui: &mut Ui,
    output: &TextEditOutput,
    find: &FindHighlights<'_>,
    palette: Palette,
    layer: ShapeIdx,
) {
    let galley = &output.galley;
    if find.reveal
        && let Some(found) = find.matches.get(find.active)
    {
        let rect = galley.pos_from_cursor(CCursor::new(found.characters.start));
        ui.scroll_to_rect(
            rect.translate(output.galley_pos.to_vec2()),
            Some(Align::Center),
        );
    }
    let clip = ui.clip_rect();
    let first = galley.cursor_from_pos(clip.min - output.galley_pos).index.0;
    let last = galley.cursor_from_pos(clip.max - output.galley_pos).index.0;
    let start = find
        .matches
        .partition_point(|found| found.characters.end < first);
    let mut shapes = Vec::new();
    for (index, found) in find.matches.iter().enumerate().skip(start) {
        if found.characters.start > last {
            break;
        }
        let first = galley.layout_from_cursor(CCursor::new(found.characters.start));
        let last = galley.layout_from_cursor(CCursor::new(found.characters.end));
        for row_index in first.row..=last.row {
            let row = &galley.rows[row_index];
            let left = if row_index == first.row {
                row.x_offset(first.column)
            } else {
                0.0
            };
            let right = if row_index == last.row {
                row.x_offset(last.column)
            } else {
                row.size.x
            };
            let rect = Rect::from_min_max(
                output.galley_pos + row.pos.to_vec2() + eframe::egui::vec2(left, 0.0),
                output.galley_pos + row.pos.to_vec2() + eframe::egui::vec2(right, row.size.y),
            );
            if ui.is_rect_visible(rect) {
                shapes.push(Shape::rect_filled(rect, 2, palette.selection));
                if index == find.active {
                    shapes.push(Shape::rect_stroke(
                        rect,
                        2,
                        Stroke::new(1.0, palette.accent),
                        StrokeKind::Outside,
                    ));
                }
            }
        }
    }
    ui.painter().set(layer, Shape::Vec(shapes));
}
