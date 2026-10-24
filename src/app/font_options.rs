//! A searchable, keyboard-accessible picker backed by the operating system font list.
use super::QuickNoteApp;
use super::settings_section::SettingsSection;
use crate::i18n;
use crate::ui;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Id;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

impl QuickNoteApp {
    pub(super) fn open_font_options(&mut self, context: &Context) {
        self.settings = Some(SettingsSection::Fonts);
        self.settings_request = None;
        self.font_focus = true;
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
        let compact = ui.ctx().content_rect().height() < 400.0;
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
        if !compact {
            let label = i18n::format(
                &self.locale.text().current_font,
                &[(
                    "font",
                    crate::appearance::font_display_name(&self.theme.typography.font_family),
                )],
            );
            ui.add(eframe::egui::Label::new(&label).truncate());
        }
        let search = ui.add_sized(
            [ui.available_width(), 32.0],
            TextEdit::singleline(&mut self.font_query)
                .id(search_id)
                .hint_text(self.locale.text().search_fonts.as_str())
                .desired_width(f32::INFINITY),
        );
        search.widget_info(|| {
            let mut info = WidgetInfo::text_edit(true, &self.font_query, &self.font_query, "");
            info.label = Some(self.locale.text().search_fonts.clone());
            info
        });
        if self.font_focus && !self.close_blocked && self.pending_delete.is_none() {
            search.request_focus();
            self.font_focus = false;
        }
        ui::focus_ring(ui, &search, palette);
        if search.changed() {
            self.font_index = 0;
        }
        if self.fonts.names().is_none() {
            ui.weak(self.locale.text().loading_fonts.as_str());
        }
        self.font_choices
            .update(self.fonts.names(), &self.font_query);
        let names = self.font_choices.names();
        self.font_index = self.font_index.min(names.len().saturating_sub(1));
        if names.is_empty() {
            ui.weak(self.locale.text().no_fonts.as_str());
        }
        let reveal = search.changed() || navigation.0 || navigation.1;
        if navigation.0 {
            self.font_index = (self.font_index + 1).min(names.len().saturating_sub(1));
        }
        if navigation.1 {
            self.font_index = self.font_index.saturating_sub(1);
        }
        let mut selected = names.get(self.font_index).filter(|_| navigation.2).cloned();
        if !compact {
            ui.weak(self.locale.text().font_keyboard_help.as_str());
        }
        let mut reset_font = false;
        ScrollArea::vertical()
            .id_salt("font-list")
            .max_height(ui.available_height().max(36.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let reset = ui.button(self.locale.text().use_theme_font.as_str());
                ui::focus_ring(ui, &reset, palette);
                reset_font = reset.clicked();
                ui.separator();
                for (index, name) in names.iter().enumerate() {
                    let response = ui.selectable_label(
                        if search.has_focus() {
                            index == self.font_index
                        } else {
                            self.theme.typography.font_family == *name
                        },
                        name,
                    );
                    ui::focus_ring(ui, &response, palette);
                    if (reveal && index == self.font_index) || response.gained_focus() {
                        response.scroll_to_me(None);
                    }
                    if response.clicked() {
                        selected = Some(name.clone());
                    }
                }
            });
        if reset_font && let Err(error) = self.set_editor_font(None, ui.ctx()) {
            self.theme_error = Some(error);
        }
        if let Some(name) = selected
            && let Err(error) = self.set_editor_font(Some(&name), ui.ctx())
        {
            self.theme_error = Some(error);
        }
    }
}
