//! Construct AppKit menus once; bindings and language changes rebuild them transactionally.
use super::EditCommand;
use super::MenuAction;
use crate::i18n;
use crate::i18n::Messages;
use crate::shortcut::Command;
use crate::shortcut::Shortcuts;
use crate::storage::Result;
use eframe::egui::Context;
use eframe::egui::Key;
use eframe::egui::KeyboardShortcut;
use eframe::egui::Modifiers;
use muda::AboutMetadata;
use muda::Menu;
use muda::MenuItem;
use muda::PredefinedMenuItem;
use muda::Submenu;

pub(super) struct BuiltMenu {
    pub root: Menu,
    pub items: Vec<(MenuItem, MenuAction, Option<String>)>,
}

impl BuiltMenu {
    pub fn new(text: &Messages, shortcuts: &Shortcuts, context: &Context) -> Result<Self> {
        let mut built = Self {
            root: Menu::new(),
            items: Vec::new(),
        };
        let app = Submenu::new(&text.app_name, true);
        app.append(&PredefinedMenuItem::about(
            Some(&i18n::format(&text.menu_about, &[("app", &text.app_name)])),
            Some(AboutMetadata {
                name: Some(text.app_name.clone()),
                version: Some(env!("CARGO_PKG_VERSION").into()),
                ..Default::default()
            }),
        ))?;
        app.append(&PredefinedMenuItem::separator())?;
        built.app(
            &app,
            &text.menu_settings,
            Command::Options,
            shortcuts,
            context,
        )?;
        app.append(&PredefinedMenuItem::separator())?;
        for (id, label, action, keys) in [
            (
                "app.hide",
                i18n::format(&text.menu_hide, &[("app", &text.app_name)]),
                MenuAction::Hide,
                Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::H)),
            ),
            (
                "app.hide-others",
                text.menu_hide_others.clone(),
                MenuAction::HideOthers,
                Some(KeyboardShortcut::new(
                    Modifiers::COMMAND | Modifiers::ALT,
                    Key::H,
                )),
            ),
            (
                "app.show-all",
                text.menu_show_all.clone(),
                MenuAction::ShowAll,
                None,
            ),
        ] {
            built.item(
                &app,
                id,
                &label,
                action,
                keys.filter(|keys| !shortcuts.assigned(*keys)),
                context,
            )?;
        }
        app.append(&PredefinedMenuItem::separator())?;
        built.item(
            &app,
            "app.quit",
            &i18n::format(&text.menu_quit, &[("app", &text.app_name)]),
            MenuAction::App(Command::Quit),
            shortcuts.keys(Command::Quit).next(),
            context,
        )?;
        let file = Submenu::new(&text.menu_file, true);
        for (label, command) in [
            (&text.new_note, Command::New),
            (&text.quick_capture, Command::Capture),
            (&text.capture.new_note, Command::NewCapture),
            (&text.capture.clipboard, Command::CaptureClipboard),
            (&text.close_note, Command::CloseTab),
            (&text.recently_deleted, Command::Trash),
            (&text.menu_save, Command::Save),
            (&text.export_note, Command::ExportNote),
        ] {
            built.app(&file, label, command, shortcuts, context)?;
        }
        file.append(&PredefinedMenuItem::separator())?;
        built.app(
            &file,
            &text.shortcut_delete,
            Command::Delete,
            shortcuts,
            context,
        )?;
        built.app(
            &file,
            &text.close_window,
            Command::Close,
            shortcuts,
            context,
        )?;
        let edit = Submenu::new(&text.edit, true);
        for (label, command) in [
            (&text.menu_undo, EditCommand::Undo),
            (&text.menu_redo, EditCommand::Redo),
            (&text.menu_cut, EditCommand::Cut),
            (&text.copy, EditCommand::Copy),
            (&text.menu_paste, EditCommand::Paste),
            (&text.menu_select_all, EditCommand::SelectAll),
        ] {
            if command == EditCommand::Cut {
                edit.append(&PredefinedMenuItem::separator())?;
            }
            let keys = command.keys();
            built.item(
                &edit,
                &format!("edit.{command:?}"),
                label,
                MenuAction::Edit(command),
                (!shortcuts.assigned(keys)).then_some(keys),
                context,
            )?;
        }
        edit.append(&PredefinedMenuItem::separator())?;
        for (label, command) in [
            (&text.find_in_note, Command::Find),
            (&text.replace_in_note, Command::Replace),
            (&text.find_next, Command::FindNext),
            (&text.find_previous, Command::FindPrevious),
            (&text.find_note, Command::Search),
        ] {
            built.app(&edit, label, command, shortcuts, context)?;
        }
        let view = Submenu::new(&text.menu_view, true);
        built.app(&view, &text.preview, Command::Preview, shortcuts, context)?;
        built.app(
            &view,
            &text.focus_mode,
            Command::FocusMode,
            shortcuts,
            context,
        )?;
        built.app(
            &view,
            &text.pin_window,
            Command::PinWindow,
            shortcuts,
            context,
        )?;
        built.app(
            &view,
            &text.shortcut_toggle_theme,
            Command::ToggleTheme,
            shortcuts,
            context,
        )?;
        built.item(
            &view,
            "view.fonts",
            &text.editor_font,
            MenuAction::Fonts,
            None,
            context,
        )?;
        let window = Submenu::new(&text.menu_window, true);
        for (label, command) in [
            (&text.shortcut_previous, Command::PreviousNote),
            (&text.shortcut_next, Command::NextNote),
            (&text.move_tab_left, Command::MoveTabLeft),
            (&text.move_tab_right, Command::MoveTabRight),
            (&text.minimize_window, Command::Minimize),
            (&text.maximize_window, Command::Maximize),
        ] {
            built.app(&window, label, command, shortcuts, context)?;
        }
        let help = Submenu::new(&text.menu_help, true);
        built.app(
            &help,
            &text.keyboard_shortcuts,
            Command::Help,
            shortcuts,
            context,
        )?;
        help.append(&PredefinedMenuItem::separator())?;
        for (label, command) in [
            (&text.open_shortcuts_folder, Command::ShortcutsFolder),
            (&text.reload_shortcuts, Command::ReloadShortcuts),
        ] {
            built.app(&help, label, command, shortcuts, context)?;
        }
        built
            .root
            .append_items(&[&app, &file, &edit, &view, &window, &help])?;
        window.set_as_windows_menu_for_nsapp();
        help.set_as_help_menu_for_nsapp();
        Ok(built)
    }
}
