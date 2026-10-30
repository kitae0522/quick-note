//! Keyboard commands are translated into the same actions as pointer input.
use super::Mode;
use super::QuickNoteApp;
use super::settings_section::SettingsSection;
use crate::ui::header::HeaderAction;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::FocusDirection;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::ViewportCommand;
use eframe::egui::text_edit::TextEditState;

impl QuickNoteApp {
    pub(super) fn keys(&mut self, context: &Context) {
        if self.pending_delete.is_some() || self.close_blocked {
            return;
        }
        let ime_owns_navigation = context.input(|input| self.composing || input.events.iter().any(|event| matches!(
            event, Event::Ime(ImeEvent::Preedit { text, .. } | ImeEvent::Commit(text)) if !text.is_empty()
        )));
        context.input(|input| {
            for event in &input.events {
                match event {
                    Event::Ime(ImeEvent::Preedit { text, .. }) => self.composing = !text.is_empty(),
                    Event::Ime(ImeEvent::Commit(_)) | Event::WindowFocused(false) => {
                        self.composing = false
                    }
                    _ => {}
                }
            }
        });
        if let Some(command) = context.input_mut(|input| self.shortcuts.consume(input)) {
            // Consumed app shortcuts must not also traverse widgets (Ctrl+Shift+Tab on macOS).
            context.memory_mut(|memory| memory.move_focus(FocusDirection::None));
            if ime_owns_navigation
                && matches!(
                    command,
                    crate::shortcut::Command::CloseTab
                        | crate::shortcut::Command::Delete
                        | crate::shortcut::Command::NewCapture
                        | crate::shortcut::Command::CaptureClipboard
                )
            {
                self.pending_menu
                    .push(crate::menu::MenuAction::App(command));
                context.request_repaint();
            } else {
                self.command(command, context);
            }
        }
        if ime_owns_navigation {
            return;
        }
        self.find_keys(context);
        let text_focused = context
            .memory(|memory| memory.focused())
            .is_some_and(|id| TextEditState::load(context, id).is_some());
        let mut selected = None;
        let escape = context.input_mut(|input| {
            if self.search_open && text_focused {
                if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                    self.search_index =
                        (self.search_index + 1).min(self.search_results.len().saturating_sub(1));
                    self.search_reveal = true;
                }
                if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                    self.search_index = self.search_index.saturating_sub(1);
                    self.search_reveal = true;
                }
                if input.consume_key(Modifiers::NONE, Key::Enter) {
                    selected = self
                        .search_results
                        .get(self.search_index)
                        .map(|note| note.id.clone());
                }
            }
            input.consume_key(Modifiers::NONE, Key::Escape)
        });
        if let Some(id) = selected
            && let Err(error) = self.open_search_result(&id, context)
        {
            self.error = Some(error);
        }
        if escape {
            if self.document_find.open {
                self.close_find();
            } else if self.search_open {
                self.search_open = false;
                self.restore_focus(context);
            } else if self.settings_open() {
                self.close_settings(context);
            } else if self.capture.compact {
                self.hide_capture(context);
            } else {
                self.mode = Mode::Edit;
                self.focus_editor = true;
            }
        }
    }

    pub(super) fn action(&mut self, action: HeaderAction, context: &Context) {
        let outcome = match action {
            HeaderAction::Select { id, focus } => self.select_note(&id, focus),
            HeaderAction::Reorder { id, index } => {
                self.finish_tab_drag();
                self.reorder_tab(&id, index)
            }
            HeaderAction::FinishTabDrag => {
                self.finish_tab_drag();
                Ok(())
            }
            HeaderAction::New => self.new_note(),
            HeaderAction::CloseTab(id) => self.close_tab(&id),
            HeaderAction::Delete(id) => {
                self.previous_focus = context.memory(|memory| memory.focused());
                self.request_delete(&id)
            }
            HeaderAction::Search => {
                self.request_search(context);
                Ok(())
            }
            HeaderAction::Options => {
                self.open_settings(SettingsSection::General, context);
                Ok(())
            }
            HeaderAction::Minimize => {
                context.send_viewport_cmd(ViewportCommand::Minimized(true));
                Ok(())
            }
            HeaderAction::Maximize => {
                let maximized = context.input(|input| input.viewport().maximized.unwrap_or(false));
                context.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                Ok(())
            }
            HeaderAction::CloseWindow => {
                context.send_viewport_cmd(ViewportCommand::Close);
                Ok(())
            }
        };
        if let Err(error) = outcome {
            self.error = Some(error);
        }
    }
}
