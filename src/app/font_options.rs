//! A searchable, keyboard-accessible picker backed by the operating system font list.
use super::QuickNoteApp;
use super::font_workspace::FontAction;
use super::settings_section::SettingsSection;
use crate::i18n;
use crate::ui;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::FontId;
use eframe::egui::Id;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

impl QuickNoteApp {
    pub(super) fn open_font_options(&mut self, context: &Context) {
        self.settings = Some(SettingsSection::Fonts);
        self.settings_request = None;
        self.font_focus = true;
        self.font_action_focus = None;
        self.font_order_focus = None;
        self.font_workspace_bounds = None;
        self.font_query.clear();
        self.font_index = 0;
        context.request_repaint();
        if let Err(error) = self.fonts.request(context) {
            self.theme_error = Some(
                i18n::message(|m| &m.error_font_list)
                    .cause("error", error)
                    .into(),
            );
        }
    }

    pub(super) fn font_options(&mut self, ui: &mut Ui) {
        let palette = self.theme.palette();
        let compact = ui.ctx().content_rect().height() < 520.0;
        let search_id = Id::unique("font-search");
        let ime_active = self.composing || ui.input(|input| input.events.iter().any(|event| {
            matches!(event, Event::Ime(ImeEvent::Preedit { text, .. } | ImeEvent::Commit(text)) if !text.is_empty())
        }));
        let navigation = if ui.memory(|memory| memory.has_focus(search_id)) && !ime_active {
            ui.input_mut(|input| {
                (
                    input.consume_key(Modifiers::NONE, Key::ArrowDown),
                    input.consume_key(Modifiers::NONE, Key::ArrowUp),
                    input.consume_key(Modifiers::NONE, Key::Enter),
                )
            })
        } else {
            (false, false, false)
        };
        let loading = self.fonts.names().is_none();
        let search = ui
            .horizontal(|ui| {
                let reserved = if loading {
                    ui.fonts_mut(|fonts| {
                        fonts
                            .layout_no_wrap(
                                self.locale.text().loading_fonts.clone(),
                                FontId::proportional(12.0),
                                palette.secondary,
                            )
                            .size()
                            .x
                    }) + ui.spacing().item_spacing.x
                } else {
                    0.0
                };
                let response = ui.add_sized(
                    [(ui.available_width() - reserved).max(32.0), 32.0],
                    TextEdit::singleline(&mut self.font_query)
                        .id(search_id)
                        .hint_text(self.locale.text().search_fonts.as_str())
                        .desired_width(f32::INFINITY),
                );
                if loading {
                    let status = ui.label(
                        RichText::new(&self.locale.text().loading_fonts)
                            .small()
                            .color(palette.secondary),
                    );
                    status.widget_info(|| {
                        WidgetInfo::labeled(
                            Role::ProgressIndicator,
                            true,
                            &self.locale.text().loading_fonts,
                        )
                    });
                    status.on_hover_text(&self.locale.text().loading_fonts);
                }
                response
            })
            .inner;
        search.widget_info(|| {
            let mut info = WidgetInfo::text_edit(true, &self.font_query, &self.font_query, "");
            info.label = Some(self.locale.text().search_fonts.clone());
            info
        });
        if (self.font_focus || self.font_action_focus == Some(search_id))
            && !self.close_blocked
            && self.pending_delete.is_none()
        {
            search.request_focus();
            self.font_focus = false;
            self.font_action_focus = None;
        }
        ui::focus_ring(ui, &search, palette);
        if search.changed() {
            self.font_index = 0;
        }
        self.font_choices
            .update(self.fonts.names(), &self.font_query);
        let names = self.font_choices.names();
        self.font_index = self.font_index.min(names.len().saturating_sub(1));
        if names.is_empty() {
            ui.weak(self.locale.text().no_fonts.as_str());
        }
        let reveal = search.changed() || navigation.0 || navigation.1 || navigation.2;
        if navigation.0 {
            self.font_index = (self.font_index + 1).min(names.len().saturating_sub(1));
        }
        if navigation.1 {
            self.font_index = self.font_index.saturating_sub(1);
        }
        let selected = names.get(self.font_index).filter(|_| navigation.2).cloned();
        if !compact {
            ui.weak(self.locale.text().font_keyboard_help.as_str());
        }
        let action = self
            .font_workspace(ui, compact, reveal, search.has_focus())
            .or_else(|| selected.map(FontAction::Toggle));
        let updated = match action {
            Some(FontAction::Reset) => {
                if let Err(error) = self.set_editor_font(None, ui.ctx()) {
                    self.theme_error = Some(error);
                }
                None
            }
            Some(FontAction::Toggle(name)) => {
                self.toggle_font_choice(&name).map(|names| (names, None))
            }
            Some(FontAction::Reorder { names, focus }) => Some((names, Some(focus))),
            None => None,
        };
        if let Some((names, focus)) = updated {
            match self.set_editor_fonts(Some(&names), ui.ctx()) {
                Ok(()) => {
                    if let Some(focus) = focus {
                        self.font_action_focus = Some(focus);
                        ui.ctx().request_repaint();
                    }
                }
                Err(error) => self.theme_error = Some(error),
            }
        }
    }
}
