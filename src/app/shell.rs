//! Coordinate window state, header, content and transient surfaces.
use super::QuickNoteApp;
use crate::ui::header;
use crate::ui::motion;
use eframe::egui::Event;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::ImeEvent;
use eframe::egui::OutputCommand;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::Ui;
use eframe::egui::ViewportCommand;
use eframe::egui::WidgetInfo;

impl QuickNoteApp {
    pub fn show(&mut self, ui: &mut Ui) {
        let context = ui.ctx().clone();
        self.poll_fonts(&context);
        let window_focused = context.input(|input| input.focused);
        if window_focused && !self.window_was_focused {
            self.reload_shortcuts();
            self.configure_motion(&context);
        }
        if window_focused
            && !self.window_was_focused
            && !matches!(self.theme_id.as_str(), "helium" | "graphite")
        {
            self.reload_theme(&context);
        }
        self.window_was_focused = window_focused;
        self.keys(&context);
        self.receive_menu(&context);
        if context.input(|input| input.viewport().close_requested())
            && self.dirty
            && !self.persist()
        {
            context.send_viewport_cmd(ViewportCommand::CancelClose);
            if !self.close_blocked {
                self.modal_new = true;
            }
            self.close_blocked = true;
        }
        let palette = self.theme.palette();
        let header_height = self.theme.layout.toolbar_height;
        let focus_before_header = context.memory(|memory| memory.focused());
        let header = Panel::top("header")
            .frame(Frame::NONE)
            .exact_size(header_height)
            .show(ui, |ui| {
                header::header(
                    ui,
                    &mut self.header,
                    &self.tabs,
                    if self.settings.is_some() {
                        ""
                    } else {
                        &self.note.id
                    },
                    palette,
                    header::WindowControls::current(),
                    self.locale.text(),
                )
            })
            .inner;
        self.search_rect = header.search_rect;
        if header.search_changed || header.search_gained_focus {
            if !self.search_open {
                self.request_search(&context);
                self.previous_focus = focus_before_header;
            }
            self.search_query = None;
        }
        if let Some(action) = header.action {
            self.action(action, &context);
        }
        if self.settings.is_none() {
            self.status_bar(ui);
        }
        if self.error.is_some() || self.theme_error.is_some() || self.shortcut_error.is_some() {
            Panel::bottom("notice")
                .frame(Frame::new().fill(palette.toolbar).inner_margin(8))
                .show(ui, |ui| {
                    let Some(message) = self
                        .error
                        .as_ref()
                        .or(self.theme_error.as_ref())
                        .or(self.shortcut_error.as_ref())
                    else {
                        return;
                    };
                    let message = self.locale.error(message.as_ref());
                    let response =
                        ui.label(RichText::new(&message).size(12.0).color(palette.foreground));
                    response.widget_info(|| WidgetInfo::labeled(Role::Alert, true, &message));
                    context.accesskit_node_builder(response.id, |node| {
                        node.set_live(eframe::egui::accesskit::Live::Assertive)
                    });
                    ui.horizontal(|ui| {
                        if self.dirty
                            && ui.button(self.locale.text().export_text.as_str()).clicked()
                        {
                            self.export_note();
                        }
                        if !self.dirty && ui.button(self.locale.text().dismiss.as_str()).clicked() {
                            self.error = None;
                            self.theme_error = None;
                            self.shortcut_error = None;
                        }
                    });
                });
        }
        let settings_opacity = motion::reveal(
            &context,
            Id::unique("settings-reveal"),
            self.settings.is_some(),
        );
        if self.settings.is_some() {
            self.settings_view(ui, settings_opacity);
        } else {
            if self.settings_request.is_some()
                && context.input(|input| {
                    input.events.iter().any(
                    |event| matches!(event, Event::Ime(ImeEvent::Commit(text)) if !text.is_empty()),
                )
                })
            {
                self.focus_editor = true;
            }
            self.find_bar(ui);
            self.document_view(ui);
            self.finish_find_navigation(&context);
            self.finish_settings_navigation(&context);
        }
        let search_opacity =
            motion::reveal(&context, Id::unique("search-reveal"), self.search_open);
        if self.search_open {
            self.search_popup(&context, search_opacity);
        }
        let delete_opacity = motion::reveal(
            &context,
            Id::unique("delete-reveal"),
            self.pending_delete.is_some(),
        );
        if self.pending_delete.is_some() {
            self.delete_dialog(&context, delete_opacity);
        }
        let close_opacity =
            motion::reveal(&context, Id::unique("close-reveal"), self.close_blocked);
        if self.close_blocked {
            self.close_dialog(&context, close_opacity);
        }
        self.update_menu(&context);
        context.output_mut(|output| {
            output.commands.retain(|command| match command {
                OutputCommand::OpenUrl(url) => ["https://", "http://", "mailto:"]
                    .iter()
                    .any(|scheme| url.url.starts_with(scheme)),
                _ => true,
            })
        });
    }
}
