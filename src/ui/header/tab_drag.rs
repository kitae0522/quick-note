//! Drag state outlives virtualized tab widgets; only a drop inside the strip commits a move.
use super::HeaderAction;
use super::HeaderState;
use super::tab_strip::TabStrip;
use crate::appearance::Palette;
use eframe::egui;
use eframe::egui::Align2;
use eframe::egui::DragAndDrop;
use eframe::egui::FontId;
use eframe::egui::Id;
use eframe::egui::LayerId;
use eframe::egui::Order;
use eframe::egui::PointerButton;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use eframe::egui::Response;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::Ui;
use eframe::egui::Vec2;

pub(super) struct TabDrag {
    id: String,
    title: String,
    width: f32,
    height: f32,
    grab_offset: Vec2,
}

impl TabDrag {
    pub fn start(response: &Response, id: &str, title: &str, width: f32) {
        if response.drag_started_by(PointerButton::Primary) {
            let origin = response.ctx.input(|input| input.pointer.press_origin());
            DragAndDrop::set_payload(
                &response.ctx,
                Self {
                    id: id.to_owned(),
                    title: title.chars().take(32).collect(),
                    width,
                    height: response.rect.height(),
                    grab_offset: origin.unwrap_or(response.rect.center()) - response.rect.min,
                },
            );
        }
    }

    pub fn is_dragging(ui: &Ui, id: &str) -> bool {
        DragAndDrop::payload::<Self>(ui.ctx()).is_some_and(|drag| drag.id == id)
    }

    fn paint(&self, ui: &Ui, position: Pos2, strip: Rect, palette: Palette) {
        let painter = ui
            .ctx()
            .layer_painter(LayerId::new(Order::Tooltip, Id::unique("tab-drag")));
        let mut origin = position - self.grab_offset;
        // Keep the placement marker and its target readable above the floating title.
        origin.y = origin.y.max(strip.bottom() + 4.0);
        let rect = Rect::from_min_size(origin, egui::vec2(self.width, self.height));
        painter.rect_filled(rect, 8, palette.background);
        painter.rect_stroke(
            rect,
            8,
            Stroke::new(1.0, palette.accent),
            StrokeKind::Inside,
        );
        painter
            .with_clip_rect(rect.shrink2(egui::vec2(8.0, 0.0)))
            .text(
                rect.center(),
                Align2::CENTER_CENTER,
                &self.title,
                FontId::proportional(13.0),
                palette.foreground,
            );
    }
}

impl TabStrip<'_> {
    pub(super) fn drag(
        &self,
        ui: &Ui,
        state: &mut HeaderState,
        rect: Rect,
    ) -> Option<HeaderAction> {
        let Some(drag) = DragAndDrop::payload::<TabDrag>(ui.ctx()) else {
            return std::mem::take(&mut state.dragging).then_some(HeaderAction::FinishTabDrag);
        };
        state.dragging = true;
        let (position, released, down, elapsed) = ui.input(|input| {
            (
                input.pointer.interact_pos(),
                input.pointer.button_released(PointerButton::Primary),
                input.pointer.primary_down(),
                input.stable_dt.min(0.05),
            )
        });
        if !ui.input(|input| input.focused) || (!down && !released) {
            state.dragging = false;
            DragAndDrop::clear_payload(ui.ctx());
            return Some(HeaderAction::FinishTabDrag);
        }
        let position = position?;
        if down {
            drag.paint(ui, position, rect, self.palette);
        }
        if !rect.contains(position) {
            if released {
                state.dragging = false;
                DragAndDrop::take_payload::<TabDrag>(ui.ctx());
                return Some(HeaderAction::FinishTabDrag);
            }
            return None;
        }
        let source = self.tabs.iter().position(|note| note.id == drag.id)?;
        let stride = self.tab_width + 2.0;
        let logical_x = position.x - rect.left() + state.scroll;
        let slot = (logical_x / stride)
            .round()
            .clamp(0.0, self.tabs.len() as f32) as usize;
        let index = slot.saturating_sub(usize::from(slot > source));
        if index != source {
            let x = (rect.left() + slot as f32 * stride - state.scroll)
                .clamp(rect.left() + 1.0, rect.right() - 1.0);
            ui.ctx()
                .layer_painter(LayerId::new(Order::Tooltip, Id::unique("tab-drag")))
                .with_clip_rect(rect)
                .vline(
                    x,
                    rect.y_range().shrink(2.0),
                    Stroke::new(2.0, self.palette.accent),
                );
        }
        if released {
            state.dragging = false;
            DragAndDrop::take_payload::<TabDrag>(ui.ctx());
            return Some(if index != source {
                HeaderAction::Reorder {
                    id: drag.id.clone(),
                    index,
                }
            } else {
                HeaderAction::FinishTabDrag
            });
        }
        if down {
            let edge = 24.0_f32.min(rect.width() / 3.0);
            let direction = if position.x < rect.left() + edge {
                -((rect.left() + edge - position.x) / edge)
            } else if position.x > rect.right() - edge {
                (position.x - rect.right() + edge) / edge
            } else {
                0.0
            };
            let maximum = (self.tabs.len() as f32 * stride - 2.0 - rect.width()).max(0.0);
            let next = (state.scroll + direction * 720.0 * elapsed).clamp(0.0, maximum);
            if next != state.scroll {
                state.scroll = next;
                ui.ctx().request_repaint();
            }
        }
        None
    }
}
