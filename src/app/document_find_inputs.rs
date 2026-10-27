//! Native text fields keep their purpose visible without adding a label row.
use super::QuickNoteApp;
use crate::ui;
use eframe::egui::Id;
use eframe::egui::RichText;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;
use eframe::egui::text::CCursor;
use eframe::egui::text::CCursorRange;
use eframe::egui::text_edit::TextEditState;

impl QuickNoteApp {
    pub(super) fn find_query_input(&mut self, ui: &mut Ui, width: f32) -> bool {
        let id = Id::unique("document-find-query");
        if std::mem::take(&mut self.document_find.focus) {
            ui.memory_mut(|memory| memory.request_focus(id));
            let mut state = TextEditState::load(ui.ctx(), id).unwrap_or_default();
            state.cursor.set_char_range(Some(CCursorRange::two(
                CCursor::new(0),
                CCursor::new(self.document_find.query.chars().count()),
            )));
            state.store(ui.ctx(), id);
        }
        let text = self.locale.text();
        let palette = self.theme.palette();
        let response = ui.add_sized(
            [width, ui::controls::CONTROL_HEIGHT],
            TextEdit::singleline(&mut self.document_find.query)
                .id(id)
                .prefix(
                    RichText::new(text.find_text.clone())
                        .size(12.0)
                        .color(palette.secondary),
                )
                .desired_width(f32::INFINITY),
        );
        response.widget_info(|| {
            let mut info = WidgetInfo::text_edit(
                true,
                &self.document_find.query,
                &self.document_find.query,
                "",
            );
            info.label = Some(text.find_in_note.clone());
            info
        });
        ui::focus_ring(ui, &response, palette);
        response.changed()
    }

    pub(super) fn find_replacement_input(&mut self, ui: &mut Ui, width: f32) {
        let text = self.locale.text();
        let palette = self.theme.palette();
        let response = ui.add_sized(
            [width, ui::controls::CONTROL_HEIGHT],
            TextEdit::singleline(&mut self.document_find.replacement)
                .id(Id::unique("document-find-replacement"))
                .prefix(
                    RichText::new(text.replace_with.clone())
                        .size(12.0)
                        .color(palette.secondary),
                )
                .desired_width(f32::INFINITY),
        );
        response.widget_info(|| {
            let mut info = WidgetInfo::text_edit(
                true,
                &self.document_find.replacement,
                &self.document_find.replacement,
                "",
            );
            info.label = Some(text.replace_with.clone());
            info
        });
        ui::focus_ring(ui, &response, palette);
    }
}
