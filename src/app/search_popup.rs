//! Search results are constrained to the window safe area.
use super::QuickNoteApp;
use crate::i18n;
use crate::ui;
use crate::ui::FocusTarget;
use crate::ui::safe_area;
use eframe::egui;
use eframe::egui::Button;
use eframe::egui::Context;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::Rect;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::WidgetInfo;
use eframe::egui::Window;

impl QuickNoteApp {
    pub(super) fn search_popup(&mut self, context: &Context, opacity: f32) {
        if let Err(error) = self.refresh_search() {
            self.error = Some(
                i18n::message(|m| &m.error_search)
                    .cause("error", error)
                    .into(),
            );
        }
        let mut safe = context
            .content_rect()
            .shrink(f32::from(safe_area::EDGE_INSET));
        safe.min.y = safe.top().max(
            context.content_rect().top()
                + self.theme.layout.toolbar_height
                + safe_area::POPOVER_GAP,
        );
        let width = 380.0_f32.min(safe.width());
        let position = egui::pos2(
            self.search_rect
                .left()
                .clamp(safe.left(), safe.right() - width),
            safe.top(),
        );
        let bounds = Rect::from_min_size(position, egui::vec2(width, safe.height().min(320.0)));
        let compact = context.content_rect().width() < 620.0;
        let mut selected = None;
        let result = Window::new(self.locale.text().find_note.as_str())
            .id(Id::unique("note-search-popup"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .frame(
                Frame::window(&context.style_of(context.theme()))
                    .inner_margin(safe_area::EDGE_INSET),
            )
            .constrain_to(safe)
            .fixed_rect(bounds)
            .show(context, |ui| {
                ui.multiply_opacity(opacity);
                if compact {
                    let id = Id::unique("compact-note-search");
                    if self.search_focus_popup || ui.memory(|memory| memory.focused().is_none()) {
                        ui.memory_mut(|memory| memory.request_focus(id));
                    }
                    let response = ui.add_sized(
                        [ui.available_width(), 28.0],
                        TextEdit::singleline(&mut self.header.query)
                            .return_key(None)
                            .id(id)
                            .hint_text(self.locale.text().search_hint.as_str()),
                    );
                    response.widget_info(|| {
                        let mut info = WidgetInfo::text_edit(
                            true,
                            &self.header.query,
                            &self.header.query,
                            self.locale.text().search_hint.as_str(),
                        );
                        info.label = Some(self.locale.text().search_notes.as_str().into());
                        info
                    });
                    ui::focus_ring(ui, &response, self.theme.palette());
                    if self.search_focus_popup {
                        response.request_focus();
                        self.search_focus_popup = !response.has_focus();
                    }
                    if response.changed() {
                        self.search_query = None;
                        context.request_repaint();
                    }
                } else {
                    ui.label(
                        RichText::new(self.locale.text().find_note.as_str())
                            .font(FontId::new(13.0, FontFamily::Name("medium".into()))),
                    );
                    if ui.memory(|memory| memory.focused().is_none()) {
                        self.header.focus_search = true;
                        context.request_repaint();
                    }
                }
                let help = ui.painter().layout(
                    self.locale.text().search_help.clone(),
                    FontId::proportional(12.0),
                    self.theme.palette().secondary,
                    ui.available_width(),
                );
                let height =
                    (ui.available_height() - help.size().y - ui.spacing().item_spacing.y).max(0.0);
                let mut scrolling = ScrollArea::vertical()
                    .id_salt("search-results")
                    .max_height(height)
                    .auto_shrink([false, true]);
                if self.search_reveal {
                    let stride = 36.0 + ui.spacing().item_spacing.y;
                    scrolling = scrolling.vertical_scroll_offset(
                        (self.search_index as f32 * stride - (height - 36.0) / 2.0).max(0.0),
                    );
                    self.search_reveal = false;
                }
                if self.search_results.is_empty() {
                    scrolling.show(ui, |ui| {
                        ui.label(i18n::format(
                            &self.locale.text().search_empty,
                            &[("query", &self.header.query)],
                        ));
                        if ui
                            .button(self.locale.text().search_clear.as_str())
                            .clicked()
                        {
                            self.header.query.clear();
                            self.search_query = None;
                        }
                    });
                } else {
                    scrolling.show_rows(ui, 36.0, self.search_results.len(), |ui, rows| {
                        for index in rows {
                            let note = &self.search_results[index];
                            let response = ui.add_sized(
                                [ui.available_width(), 36.0],
                                Button::new(crate::ui::note_title(note, self.locale.text()))
                                    .right_text(
                                        RichText::new(if index == self.search_index {
                                            ui::icons::CHECK
                                        } else {
                                            ""
                                        })
                                        .font(FontId::new(14.0, FontFamily::Name("icons".into()))),
                                    )
                                    .truncate()
                                    .selected(index == self.search_index)
                                    .frame_when_inactive(index == self.search_index),
                            );
                            response.widget_info(|| {
                                WidgetInfo::selected(
                                    Role::Button,
                                    true,
                                    index == self.search_index,
                                    i18n::format(
                                        &self.locale.text().open_note,
                                        &[(
                                            "title",
                                            crate::ui::note_title(note, self.locale.text()),
                                        )],
                                    ),
                                )
                            });
                            if response.clicked() {
                                selected = Some(note.id.clone());
                            }
                            ui::focus_ring(ui, &response, self.theme.palette());
                            response.on_hover_text(crate::ui::note_title(note, self.locale.text()));
                        }
                    });
                }
                ui.add(Label::new(help));
            });
        if let Some(id) = selected
            && let Err(error) = self.select_note(&id, FocusTarget::Editor)
        {
            self.error = Some(error);
        }
        if let Some(result) = result {
            let outside = context.input(|input| {
                input.pointer.any_pressed()
                    && input.pointer.interact_pos().is_some_and(|position| {
                        !result.response.rect.contains(position)
                            && !self.search_rect.contains(position)
                    })
            });
            if outside {
                self.search_open = false;
            }
        }
    }
}
