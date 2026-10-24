//! Lay out only tabs intersecting the horizontal viewport, retaining stable note IDs.
use super::HeaderAction;
use super::HeaderState;
use super::note_tab::NoteTab;
use crate::appearance::Palette;
use crate::i18n::Messages;
use crate::model::NoteSummary;
use eframe::egui;
use eframe::egui::Align;
use eframe::egui::Layout;
use eframe::egui::Rect;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::scroll_area::ScrollSource;

pub(super) struct TabStrip<'note> {
    pub tabs: &'note [NoteSummary],
    pub active: &'note str,
    pub width: f32,
    pub tab_width: f32,
    pub height: f32,
    pub palette: Palette,
    pub text: &'note Messages,
}

impl TabStrip<'_> {
    pub fn show(&self, ui: &mut Ui, state: &mut HeaderState) -> Option<HeaderAction> {
        // Adding/deleting a tab changes the old scroll bounds. A keyboard reveal must not
        // start an elastic spring against those bounds before the new content is measured.
        ui.spacing_mut().scroll.kinetic.rubber_band = false;
        let stride = self.tab_width + 2.0;
        let total = (self.tabs.len() as f32 * stride - 2.0).max(0.0);
        let maximum = (total - self.width).max(0.0);
        if state.reveal_active || state.focus_tab.is_some() {
            let target = state.focus_tab.as_deref().unwrap_or(self.active);
            if let Some(index) = self.tabs.iter().position(|note| note.id == target) {
                state.scroll = (index as f32 * stride - (self.width - self.tab_width) / 2.0)
                    .clamp(0.0, maximum);
            }
            state.reveal_active = false;
        }
        state.scroll = state.scroll.clamp(0.0, maximum);
        let mut action = None;
        let scrolling = ScrollArea::horizontal()
            .id_salt("note-tabs")
            .max_width(self.width)
            .max_height(self.height)
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .scroll_source(ScrollSource::MOUSE_WHEEL)
            .horizontal_scroll_offset(state.scroll)
            .show_viewport(ui, |ui, viewport| {
                ui.set_min_size(egui::vec2(total, self.height));
                if self.tabs.is_empty() {
                    ui.label(
                        RichText::new(self.text.no_notes.as_str())
                            .color(self.palette.secondary)
                            .size(13.0),
                    );
                    return;
                }
                let first = (viewport.left() / stride).floor() as usize;
                let end = ((viewport.right() / stride).ceil() as usize + 1).min(self.tabs.len());
                let bounds = Rect::from_min_size(
                    ui.max_rect().min + egui::vec2(first as f32 * stride, 0.0),
                    egui::vec2((end.saturating_sub(first)) as f32 * stride, self.height),
                );
                ui.scope_builder(
                    UiBuilder::new()
                        .max_rect(bounds)
                        .layout(Layout::left_to_right(Align::Center)),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for index in first..end {
                            let note = &self.tabs[index];
                            let tab = NoteTab {
                                note,
                                index,
                                tabs: self.tabs,
                                width: self.tab_width,
                                height: self.height,
                                selected: note.id == self.active,
                            };
                            if let Some(selected) = tab.show(ui, state, self.palette, self.text) {
                                action = Some(selected);
                            }
                        }
                    },
                );
            });
        state.scroll = scrolling.state.clamped_offset().x;
        if let Some(reordered) = self.drag(ui, state, scrolling.inner_rect) {
            action = Some(reordered);
        }
        if action.is_some() {
            ui.ctx().request_repaint();
        }
        action
    }
}
