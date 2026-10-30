//! Native commands enter the same guarded command and text-editing paths as keyboard input.
use super::QuickNoteApp;
use super::settings_section::SettingsSection;
#[cfg(target_os = "macos")]
use crate::i18n;
use crate::menu::EditCommand;
use crate::menu::MenuAction;
use crate::menu::MenuState;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::ViewportCommand;
use eframe::egui::text_edit::TextEditState;

impl QuickNoteApp {
    /// Installs the system menu from a native app-creation callback on the main thread.
    pub fn install_menu(&mut self, context: &Context) {
        #[cfg(target_os = "macos")]
        if self.native_menu.is_none() {
            match crate::menu::MacMenu::new(self.locale.text(), &self.shortcuts, context) {
                Ok(menu) => self.native_menu = Some(menu),
                Err(error) => {
                    self.theme_error = Some(
                        i18n::message(|m| &m.error_menu)
                            .cause("error", error)
                            .into(),
                    )
                }
            }
            self.update_menu(context);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = context;
    }

    pub(super) fn invalidate_menu(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.menu_dirty = true;
        }
    }

    pub(super) fn update_menu(&mut self, context: &Context) {
        #[cfg(target_os = "macos")]
        {
            let state = self.menu_state(context);
            if let Some(menu) = &mut self.native_menu {
                if self.menu_dirty {
                    self.menu_dirty = false;
                    if let Err(error) = menu.rebuild(self.locale.text(), &self.shortcuts, context) {
                        self.theme_error = Some(
                            i18n::message(|m| &m.error_menu)
                                .cause("error", error)
                                .into(),
                        );
                    }
                }
                menu.sync(state, self.locale.text());
            }
        }
        #[cfg(not(target_os = "macos"))]
        let _ = context;
    }

    pub(super) fn receive_menu(&mut self, context: &Context, defer_edit: bool) {
        #[cfg(target_os = "macos")]
        if let Some(menu) = &self.native_menu {
            self.pending_menu.extend(menu.receive());
        }
        // Let TextEdit finish an IME commit before a menu can switch or delete its note.
        if self.composing
            || context.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, Event::Ime(_)))
            })
        {
            if !self.pending_menu.is_empty() {
                context.request_repaint();
            }
            return;
        }
        for action in std::mem::take(&mut self.pending_menu) {
            if defer_edit && matches!(action, MenuAction::Edit(_)) {
                self.pending_menu.push(action);
            } else {
                self.perform_menu_action(action, context);
            }
        }
    }

    fn menu_state(&self, context: &Context) -> MenuState {
        MenuState {
            blocked: self.pending_delete.is_some() || self.close_blocked,
            composing: self.composing,
            editing: context.input(|input| input.focused)
                && context
                    .memory(|memory| memory.focused())
                    .is_some_and(|id| TextEditState::load(context, id).is_some()),
            markdown: self.markdown_enabled,
            #[cfg(target_os = "macos")]
            preview: self.mode == super::Mode::Preview,
            #[cfg(target_os = "macos")]
            settings: self.settings_open(),
            multiple_notes: self.tabs.len() > 1,
        }
    }

    /// Queues a menu choice for processing after global shortcuts and pending IME commits.
    pub fn queue_menu_action(&mut self, action: MenuAction, context: &Context) {
        self.pending_menu.push(action);
        context.request_repaint();
    }

    fn perform_menu_action(&mut self, action: MenuAction, context: &Context) {
        if !action.enabled(self.menu_state(context)) {
            return;
        }
        match action {
            #[cfg(target_os = "macos")]
            MenuAction::Hide | MenuAction::HideOthers | MenuAction::ShowAll => {
                crate::menu::MacMenu::visibility(action)
            }
            MenuAction::OpenNote(id) => {
                if let Err(error) = self.select_note(&id, crate::ui::FocusTarget::Editor) {
                    self.error = Some(error);
                }
                self.expand_capture(context);
                Self::show_window(context);
            }
            MenuAction::App(command) => {
                if command != crate::shortcut::Command::CaptureClipboard {
                    Self::show_window(context);
                }
                if command == crate::shortcut::Command::Capture {
                    self.toggle_capture(context, false);
                } else {
                    self.command(command, context);
                }
            }
            MenuAction::Fonts => {
                self.open_settings(SettingsSection::Fonts, context);
                self.open_font_options(context);
            }
            MenuAction::Edit(command) => match command {
                EditCommand::Cut => context.send_viewport_cmd(ViewportCommand::RequestCut),
                EditCommand::Copy => context.send_viewport_cmd(ViewportCommand::RequestCopy),
                EditCommand::Paste => context.send_viewport_cmd(ViewportCommand::RequestPaste),
                command => {
                    let keys = command.keys();
                    context.input_mut(|input| {
                        input.events.push(Event::Key {
                            key: keys.logical_key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: keys.modifiers,
                        })
                    });
                }
            },
        }
        context.request_repaint();
    }
}
