//! Edit the ordered selection independently of the installed-font search results.
use super::QuickNoteApp;
use crate::appearance;
use crate::i18n;
use crate::ui;
use eframe::egui::Align;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::Role;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::WidgetInfo;

#[derive(Clone, Copy, PartialEq, Eq)]
enum OrderAction {
    Up,
    Down,
    Remove,
}

impl QuickNoteApp {
    pub(super) fn font_choice_index(&self, name: &str) -> Option<usize> {
        std::iter::once(&self.theme.typography.font_family)
            .chain(self.theme.typography.font_fallbacks.iter())
            .position(|chosen| {
                appearance::font_display_name(chosen)
                    .eq_ignore_ascii_case(appearance::font_display_name(name))
            })
    }

    pub(super) fn toggle_font_choice(&self, name: &str) -> Option<Vec<String>> {
        let mut names = self.theme.typography.font_families();
        if let Some(index) = self.font_choice_index(name) {
            if names.len() == 1 {
                return None;
            }
            names.remove(index);
        } else {
            if names.len() >= 8 || !self.font_available(name) {
                return None;
            }
            names.push(name.to_owned());
        }
        Some(names)
    }

    pub(super) fn font_stack_controls(&mut self, ui: &mut Ui) -> Option<(Vec<String>, Id)> {
        let focus = self.font_action_focus.take();
        let names = self.theme.typography.font_families();
        let text = self.locale.text();
        let palette = self.theme.palette();
        let mut changed = None;
        let mut focused = None;
        for (index, name) in names.iter().enumerate() {
            let id = Id::unique(("font-order", name));
            ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized([16.0, 32.0], Label::new((index + 1).to_string()));
                    let display = appearance::font_display_name(name);
                    let unavailable = self.fonts.names().is_some() && !self.font_available(name);
                    let label = if unavailable {
                        format!("{display} · {}", text.font_unavailable)
                    } else {
                        display.into()
                    };
                    let actions_width = 96.0 + ui.spacing().item_spacing.x * 3.0;
                    let width = (ui.available_width() - actions_width).max(24.0);
                    ui.allocate_ui_with_layout(
                        eframe::egui::vec2(width, 32.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_min_width(width);
                            ui.add(Label::new(&label).truncate()).on_hover_text(&label);
                        },
                    );
                    let mut buttons = Vec::with_capacity(3);
                    for (action, glyph, enabled, template) in [
                        (
                            OrderAction::Up,
                            ui::icons::UP,
                            index > 0,
                            &text.font_move_up,
                        ),
                        (
                            OrderAction::Down,
                            ui::icons::DOWN,
                            index + 1 < names.len(),
                            &text.font_move_down,
                        ),
                        (
                            OrderAction::Remove,
                            ui::icons::CLOSE,
                            names.len() > 1,
                            &text.font_remove,
                        ),
                    ] {
                        let label = i18n::format(template, &[("font", display)]);
                        let response = ui
                            .add_enabled_ui(enabled, |ui| {
                                ui::controls::icon_button(ui, glyph, &label, palette)
                            })
                            .inner;
                        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, &label));
                        let response = if action == OrderAction::Remove && !enabled {
                            ui.ctx().accesskit_node_builder(response.id, |node| {
                                node.set_description(&text.font_keep_one)
                            });
                            response.on_disabled_hover_ui(|ui| {
                                ui.set_max_width(ui.spacing().tooltip_width);
                                ui.label(&label);
                                ui.label(&text.font_keep_one);
                            })
                        } else {
                            response
                        };
                        if focus == Some(response.id) {
                            // The reordered row now has its final geometry, so reveal that position.
                            response.request_focus();
                            response.scroll_to_me(None);
                            ui::focus_ring(ui, &response, palette);
                        }
                        if response.has_focus() {
                            focused = Some(response.id);
                        }
                        buttons.push((action, response));
                    }
                    if let Some((action, _)) =
                        buttons.iter().find(|(_, response)| response.clicked())
                    {
                        let mut updated = names.clone();
                        match action {
                            OrderAction::Up => updated.swap(index, index - 1),
                            OrderAction::Down => updated.swap(index, index + 1),
                            OrderAction::Remove => {
                                updated.remove(index);
                            }
                        }
                        let next_action = match action {
                            OrderAction::Up if index == 1 => OrderAction::Down,
                            OrderAction::Down if index + 2 == names.len() => OrderAction::Up,
                            action => *action,
                        };
                        let focus = if *action == OrderAction::Remove {
                            Id::unique("font-search")
                        } else {
                            buttons
                                .iter()
                                .find(|(action, _)| *action == next_action)
                                .expect("all three font actions are rendered above")
                                .1
                                .id
                        };
                        changed = Some((updated, focus));
                    }
                });
            });
        }
        self.font_order_focus = focused.or(self.font_order_focus);
        changed
    }
}
