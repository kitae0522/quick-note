//! Source editing, cursor commits and Markdown presentation.
use super::Mode;
use super::QuickNoteApp;
use crate::i18n;
use crate::ui::editor;
use eframe::egui::CentralPanel;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Stroke;
use eframe::egui::TextStyle;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use egui_commonmark::CommonMarkScrollOptions;
use egui_commonmark::CommonMarkViewer;

impl QuickNoteApp {
    pub(super) fn document_view(&mut self, ui: &mut Ui) {
        if !self.markdown_enabled && self.mode == Mode::Preview {
            self.mode = Mode::Edit;
            self.focus_editor = true;
            self.restore_cursor = true;
        }
        let palette = self.theme.palette();
        CentralPanel::default()
            .frame(Frame::new().fill(palette.background))
            .show(ui, |ui| {
                if self.mode == Mode::Edit {
                    let result = editor::editor(
                        ui,
                        &mut self.note,
                        &mut self.editor_metrics,
                        &self.theme,
                        &mut self.focus_editor,
                        &mut self.restore_cursor,
                        self.locale.text(),
                    );
                    self.location = result.location;
                    if result.changed {
                        self.markdown_cache = None;
                        self.dirty = true;
                        let _saved = self.persist();
                        self.editor_metrics.saved(&self.note);
                    } else if result.selection_changed
                        && let Err(error) = self.store.save_selection(&self.note)
                    {
                        self.error = Some(
                            i18n::message(|m| &m.error_cursor_save)
                                .cause("error", error)
                                .into(),
                        );
                    }
                } else {
                    // The Markdown viewer paints its own read-only code-block frame.
                    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
                    let bounds = ui.available_rect_before_wrap();
                    let inset = ((bounds.width() - 688.0) / 2.0).max(0.0);
                    ui.style_mut()
                        .text_styles
                        .insert(TextStyle::Body, FontId::proportional(16.0));
                    ui.style_mut().spacing.extra_text_line_spacing = 6.0;
                    let alerts = self.markdown_alerts();
                    let cache = self.markdown_cache.get_or_insert_with(Default::default);
                    ui.scope_builder(
                        UiBuilder::new().max_rect(bounds.shrink2(eframe::egui::vec2(inset, 0.0))),
                        |ui| {
                            CommonMarkViewer::new().alerts(alerts).show_scrollable(
                                Id::unique(("markdown", &self.note.id)),
                                ui,
                                cache,
                                &CommonMarkScrollOptions::default()
                                    .viewport_cache(true)
                                    .content_margin(24),
                                &self.note.body,
                            );
                        },
                    );
                }
            });
    }
}
