//! Keep comparison and selection context visible while each font list scrolls independently.
use super::QuickNoteApp;
use crate::i18n;
use crate::ui;
use eframe::egui::CollapsingHeader;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::collapsing_header::CollapsingState;

pub(super) enum FontAction {
    Reset,
    Toggle(String),
    Reorder { names: Vec<String>, focus: Id },
}

impl QuickNoteApp {
    pub(super) fn font_workspace(
        &mut self,
        ui: &mut Ui,
        compact: bool,
        reveal: bool,
        search_focused: bool,
    ) -> Option<FontAction> {
        let stacked = compact || ui.available_width() < 560.0;
        self.update_font_layout_focus(ui, stacked);
        if self.theme.typography.font_fallbacks.len() == 7 {
            ui.label(&self.locale.text().font_limit_reached);
        }
        if !compact {
            self.font_preview(ui);
            ui.add_space(8.0);
        }
        if stacked {
            return self.font_stacked_workspace(ui, compact, reveal, search_focused);
        }
        ui.scope(|ui| {
            let spacing = ui.spacing().item_spacing;
            ui.spacing_mut().item_spacing.x = 24.0;
            ui.columns(2, |columns| {
                for column in columns.iter_mut() {
                    column.spacing_mut().item_spacing = spacing;
                }
                let action = self.font_order_panel(&mut columns[0]);
                let choice = self.font_candidates(&mut columns[1], reveal, search_focused, true);
                action.or(choice)
            })
        })
        .inner
    }

    fn font_stacked_workspace(
        &mut self,
        ui: &mut Ui,
        compact: bool,
        reveal: bool,
        search_focused: bool,
    ) -> Option<FontAction> {
        let minimum = ui.spacing().interact_size.y
            + 2.0 * (ui::controls::CONTROL_HEIGHT + ui.spacing().item_spacing.y);
        let scroll_together = ui.available_height() < minimum;
        let mut content = |ui: &mut Ui| {
            let count = self.theme.typography.font_fallbacks.len() + 1;
            let title = i18n::format(
                &self.locale.text().font_order_summary,
                &[("count", &count.to_string())],
            );
            let order = ui
                .scope_builder(
                    UiBuilder::new().scope_id(Id::unique("font-order-section")),
                    |ui| {
                        CollapsingHeader::new(title)
                            .id_salt("compact-font-order")
                            .default_open(!compact)
                            .open(self.font_action_focus.map(|_| true))
                            .show(ui, |ui| {
                                if scroll_together {
                                    return self.font_order_content(ui, compact);
                                }
                                let candidate_height =
                                    ui::controls::CONTROL_HEIGHT + ui.spacing().item_spacing.y;
                                let height =
                                    (ui.available_height() - candidate_height).clamp(32.0, 160.0);
                                ScrollArea::vertical()
                                    .id_salt("font-order")
                                    .min_scrolled_height(ui::controls::CONTROL_HEIGHT)
                                    .max_height(height)
                                    .show(ui, |ui| self.font_order_content(ui, compact))
                                    .inner
                            })
                    },
                )
                .inner;
            ui::focus_ring(ui, &order.header_response, self.theme.palette());
            self.font_workspace_header = Some(order.header_response.id);
            let candidate = if scroll_together {
                self.font_picker_items(ui, reveal, search_focused)
                    .map(FontAction::Toggle)
            } else {
                self.font_candidates(ui, reveal, search_focused, false)
            };
            order.body_returned.flatten().or(candidate)
        };
        if scroll_together {
            ScrollArea::vertical()
                .id_salt("font-workspace")
                .min_scrolled_height(ui::controls::CONTROL_HEIGHT)
                .auto_shrink([false, false])
                .show(ui, content)
                .inner
        } else {
            content(ui)
        }
    }

    fn font_order_content(&mut self, ui: &mut Ui, compact: bool) -> Option<FontAction> {
        if self.theme.typography.font_fallbacks.is_empty() {
            ui.weak(&self.locale.text().font_keep_one);
        }
        let action = self.font_order_reset(ui);
        let reordered = self
            .font_stack_controls(ui)
            .map(|(names, focus)| FontAction::Reorder { names, focus });
        if compact {
            self.font_preview(ui);
        }
        action.or(reordered)
    }

    fn update_font_layout_focus(&mut self, ui: &Ui, stacked: bool) {
        let bounds = ui.ctx().content_rect();
        let resized = self.font_workspace_bounds != Some(bounds);
        self.font_workspace_bounds = Some(bounds);
        let focus = self.font_action_focus.or_else(|| {
            self.font_order_focus
                .filter(|id| ui.memory(|memory| memory.has_focus(*id)))
        });
        if resized && let Some(focus) = focus {
            if stacked && let Some(header) = self.font_workspace_header {
                let mut state = CollapsingState::load_with_default_open(ui.ctx(), header, true);
                state.set_open(true);
                state.store(ui.ctx());
                // Layout changes reveal the working control immediately; manual toggles still animate.
                let _ = ui.ctx().animate_bool_with_time(header, true, 0.0);
            }
            self.font_action_focus = Some(focus);
        }
        if !stacked
            && self
                .font_workspace_header
                .is_some_and(|id| ui.memory(|memory| memory.has_focus(id)))
        {
            ui.memory_mut(|memory| memory.request_focus(Id::unique("font-search")));
            ui.ctx().request_repaint();
        }
        self.font_order_focus = None;
    }

    fn font_order_reset(&mut self, ui: &mut Ui) -> Option<FontAction> {
        let response = ui
            .scope_builder(
                UiBuilder::new().scope_id(Id::unique("font-order-reset")),
                |ui| ui.button(&self.locale.text().use_theme_font),
            )
            .inner;
        if self.font_action_focus == Some(response.id) {
            response.request_focus();
            response.scroll_to_me(None);
            self.font_action_focus = None;
        }
        if response.has_focus() {
            self.font_order_focus = Some(response.id);
        }
        ui::focus_ring(ui, &response, self.theme.palette());
        response.clicked().then_some(FontAction::Reset)
    }

    fn font_order_panel(&mut self, ui: &mut Ui) -> Option<FontAction> {
        let count = self.theme.typography.font_fallbacks.len() + 1;
        let text = self.locale.text();
        ui.horizontal(|ui| {
            ui.strong(&text.font_order);
            ui.weak(i18n::format(
                &text.font_selected_count,
                &[("count", &count.to_string())],
            ));
        });
        let action = self.font_order_reset(ui);
        ui.weak(&self.locale.text().font_order_help);
        if count == 1 {
            ui.weak(&self.locale.text().font_keep_one);
        }
        let reordered = ScrollArea::vertical()
            .id_salt("font-order")
            .min_scrolled_height(ui::controls::CONTROL_HEIGHT)
            .max_height(ui.available_height().max(32.0))
            .auto_shrink([false, false])
            .show(ui, |ui| self.font_stack_controls(ui))
            .inner
            .map(|(names, focus)| FontAction::Reorder { names, focus });
        action.or(reordered)
    }

    fn font_candidates(
        &self,
        ui: &mut Ui,
        reveal: bool,
        search_focused: bool,
        heading: bool,
    ) -> Option<FontAction> {
        if heading {
            ui.horizontal(|ui| ui.strong(&self.locale.text().font_available));
        }
        ScrollArea::vertical()
            .id_salt("font-list")
            .min_scrolled_height(ui::controls::CONTROL_HEIGHT)
            .max_height(ui.available_height().max(32.0))
            .auto_shrink([false, false])
            .show(ui, |ui| self.font_picker_items(ui, reveal, search_focused))
            .inner
            .map(FontAction::Toggle)
    }
}
