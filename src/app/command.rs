//! Execute keyboard commands through the existing note, theme and window actions.
use super::Mode;
use super::QuickNoteApp;
use super::settings_section::SettingsSection;
use super::theme_files::ThemeAction;
use crate::shortcut::Command;
use crate::ui::FocusTarget;
use crate::ui::header::HeaderAction;
use eframe::egui::Context;
use eframe::egui::Id;

impl QuickNoteApp {
    pub(super) fn command(&mut self, command: Command, context: &Context) {
        let action = match command {
            Command::New => Some(HeaderAction::New),
            Command::CloseTab if self.settings_open() => {
                self.close_settings(context);
                None
            }
            Command::CloseTab => Some(HeaderAction::CloseTab(self.note.id.clone())),
            Command::Delete => Some(HeaderAction::Delete(self.note.id.clone())),
            Command::Trash => {
                self.request_search(context);
                self.search_deleted = true;
                self.header.query.clear();
                None
            }
            Command::Capture => {
                self.toggle_capture(context, context.input(|input| input.focused));
                None
            }
            Command::NewCapture => {
                if let Err(error) = self.new_capture(context) {
                    self.error.get_or_insert(error);
                }
                None
            }
            Command::CaptureClipboard => {
                self.capture_clipboard(context);
                None
            }
            Command::FocusMode => {
                self.toggle_focus_mode(context);
                None
            }
            Command::PinWindow => {
                self.toggle_pin(context);
                None
            }
            Command::Close => Some(HeaderAction::CloseWindow),
            Command::Quit => {
                self.capture.quitting = true;
                if self.dirty && !self.persist() {
                    self.close_blocked = true;
                    self.modal_new = true;
                    Self::show_window(context);
                    None
                } else {
                    Some(HeaderAction::CloseWindow)
                }
            }
            Command::Search => Some(HeaderAction::Search),
            Command::Find | Command::Replace | Command::FindNext | Command::FindPrevious => {
                self.find_command(command, context);
                None
            }
            Command::Options | Command::Help => {
                self.open_settings(
                    if matches!(command, Command::Help) {
                        SettingsSection::Shortcuts
                    } else {
                        SettingsSection::General
                    },
                    context,
                );
                None
            }
            Command::Preview if !self.markdown_enabled => None,
            Command::Preview => {
                self.document_find.close();
                self.mode = if self.mode == Mode::Edit {
                    Mode::Preview
                } else {
                    Mode::Edit
                };
                self.search_open = false;
                self.settings = None;
                self.settings_request = None;
                self.focus_editor = self.mode == Mode::Edit;
                None
            }
            Command::Save => {
                let _saved = self.persist();
                None
            }
            Command::ExportNote => {
                self.export_note();
                None
            }
            Command::ReloadTheme => {
                self.reload_theme(context);
                None
            }
            Command::ReloadShortcuts => {
                self.reload_shortcuts();
                None
            }
            Command::ShortcutsFolder => {
                self.open_shortcuts_folder();
                None
            }
            Command::ImportTheme => {
                self.theme_action(ThemeAction::ImportTheme, context);
                None
            }
            Command::ExportTheme => {
                self.theme_action(ThemeAction::ExportTheme, context);
                None
            }
            Command::ThemesFolder => {
                self.theme_action(ThemeAction::OpenThemesFolder, context);
                None
            }
            Command::ToggleTheme => {
                let id = if self.theme.appearance == "dark" {
                    "helium"
                } else {
                    "graphite"
                };
                if let Err(error) = self.set_theme(id, context) {
                    self.theme_error = Some(error);
                }
                None
            }
            Command::NextNote | Command::PreviousNote => self
                .tabs
                .iter()
                .position(|note| note.id == self.note.id)
                .map(|index| {
                    let next = if matches!(command, Command::NextNote) {
                        index + 1
                    } else {
                        index + self.tabs.len() - 1
                    } % self.tabs.len();
                    HeaderAction::Select {
                        id: self.tabs[next].id.clone(),
                        focus: FocusTarget::Editor,
                    }
                }),
            Command::SelectNote(index) => {
                let index = if index == usize::MAX {
                    self.tabs.len().saturating_sub(1)
                } else {
                    index
                };
                self.tabs.get(index).map(|note| HeaderAction::Select {
                    id: note.id.clone(),
                    focus: FocusTarget::Editor,
                })
            }
            Command::MoveTabLeft | Command::MoveTabRight => {
                self.move_active_tab(command == Command::MoveTabRight);
                None
            }
            Command::CycleFocus => {
                let editor = Id::unique(("note-editor", &self.note.id));
                if context.memory(|memory| memory.has_focus(editor)) {
                    if self.quiet_editor() {
                        self.header.focus_search = true;
                    } else {
                        self.header.focus_tab = Some(self.note.id.clone());
                    }
                    self.header.reveal_active = true;
                } else {
                    self.mode = Mode::Edit;
                    self.search_open = false;
                    self.settings = None;
                    self.settings_request = None;
                    self.focus_editor = true;
                }
                None
            }
            Command::Minimize => Some(HeaderAction::Minimize),
            Command::Maximize => Some(HeaderAction::Maximize),
        };
        if let Some(action) = action {
            self.action(action, context);
        }
    }
}
