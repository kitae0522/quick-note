//! Construct AppKit menus once; bindings and language changes rebuild them transactionally.
use super::EditCommand;
use super::MenuAction;
use super::accelerator;
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
    fn item(
        &mut self,
        menu: &Submenu,
        id: &str,
        label: &str,
        action: MenuAction,
        keys: Option<KeyboardShortcut>,
        context: &Context,
    ) -> Result<()> {
        let item = MenuItem::with_id(id, label, true, None);
        let hint = if let Some(keys) = keys {
            let applied = accelerator::native(keys)
                .is_some_and(|native| item.set_key_accelerator(Some(native)).is_ok());
            if applied {
                None
            } else {
                // Muda retains the attempted value on error; clear it before appending the item.
                item.set_key_accelerator(None)?;
                Some(context.format_shortcut(&keys))
            }
        } else {
            None
        };
        if let Some(hint) = &hint {
            item.set_text(format!("{label} ({hint})"));
        }
        menu.append(&item)?;
        self.items.push((item, action, hint));
        Ok(())
    }

    fn app(
        &mut self,
        menu: &Submenu,
        label: &str,
        command: Command,
        shortcuts: &Shortcuts,
        context: &Context,
    ) -> Result<()> {
        let id = format!("app.{command:?}");
        self.item(
            menu,
            &id,
            label,
            MenuAction::App(command),
            shortcuts.keys(command).next(),
            context,
        )
    }

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
            MenuAction::App(Command::Close),
            shortcuts.keys(Command::Close).nth(1),
            context,
        )?;
        let file = Submenu::new(&text.menu_file, true);
        for (label, command) in [
            (&text.new_note, Command::New),
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
        let view = Submenu::new(&text.menu_view, true);
        built.app(&view, &text.preview, Command::Preview, shortcuts, context)?;
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
