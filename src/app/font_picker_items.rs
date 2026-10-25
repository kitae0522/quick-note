//! Keep installed-font choices aligned, with visible source and disabled-state explanations.
use super::QuickNoteApp;
use crate::ui;
use eframe::egui::Align2;
use eframe::egui::AtomExt;
use eframe::egui::Atoms;
use eframe::egui::Checkbox;
use eframe::egui::FontId;
use eframe::egui::Id;
use eframe::egui::Rect;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::WidgetInfo;

impl QuickNoteApp {
    pub(super) fn font_picker_items(
        &self,
        ui: &mut Ui,
        reveal: bool,
        search_focused: bool,
    ) -> Option<String> {
        let palette = self.theme.palette();
        let bounds = ui.clip_rect();
        let resized = ui.data_mut(|data| {
            let id = Id::unique("font-candidate-viewport");
            let previous = data.get_temp::<Rect>(id);
            data.insert_temp(id, bounds);
            previous != Some(bounds)
        });
        let mut selected = None;
        for (index, name) in self.font_choices.names().iter().enumerate() {
            let mut checked = self.font_choice_index(name).is_some();
            let enabled = if checked {
                !self.theme.typography.font_fallbacks.is_empty()
            } else {
                self.theme.typography.font_fallbacks.len() < 7 && self.font_available(name)
            };
            let source = self.font_source_label(name);
            let last_font = checked && !enabled;
            let description = if last_font {
                &self.locale.text().font_keep_one
            } else {
                source
            };
            let id = Id::unique(("font-choice", name));
            let response = ui
                .scope_builder(UiBuilder::new().scope_id(id), |ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                        let atoms = Atoms::new(
                            name.as_str()
                                .atom_grow(true)
                                .atom_shrink(true)
                                .atom_align(Align2::LEFT_CENTER),
                        );
                        let source_width = ui.fonts_mut(|fonts| {
                            fonts
                                .layout_no_wrap(
                                    source.into(),
                                    FontId::proportional(12.0),
                                    palette.secondary,
                                )
                                .size()
                                .x
                        });
                        let width =
                            (ui.available_width() - source_width - ui.spacing().item_spacing.x)
                                .max(32.0);
                        let checkbox = ui
                            .add_enabled_ui(enabled, |ui| {
                                ui.add_sized([width, 32.0], Checkbox::new(&mut checked, atoms))
                            })
                            .inner;
                        // The explanation stays readable even when the choice itself is disabled.
                        ui.label(RichText::new(source).small().color(palette.secondary));
                        checkbox
                    })
                    .inner
                })
                .inner;
            response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, checked, name));
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_description(description));
            if search_focused && index == self.font_index {
                ui.ctx()
                    .accesskit_node_builder(Id::unique("font-search"), |node| {
                        node.set_controls([ui.unique_id().accesskit_id()]);
                        node.set_active_descendant(response.id.accesskit_id());
                    });
                ui.painter().rect_stroke(
                    response.rect.expand(1.0),
                    4,
                    Stroke::new(2.0, palette.accent),
                    StrokeKind::Inside,
                );
            }
            ui::focus_ring(ui, &response, palette);
            if (reveal && index == self.font_index)
                || (resized && response.has_focus())
                || response.gained_focus()
            {
                response.scroll_to_me(None);
            }
            if response.clicked() {
                selected = Some(name.clone());
            }
            response.on_hover_text(name).on_disabled_hover_ui(|ui| {
                ui.set_max_width(ui.spacing().tooltip_width);
                ui.label(name);
                if last_font {
                    ui.label(&self.locale.text().font_keep_one);
                }
            });
        }
        selected
    }
}
