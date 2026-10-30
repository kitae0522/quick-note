//! Shared construction for native menu items and their configured accelerators.
use super::MenuAction;
use super::accelerator;
use super::builder::BuiltMenu;
use crate::shortcut::Command;
use crate::shortcut::Shortcuts;
use crate::storage::Result;
use eframe::egui::Context;
use eframe::egui::KeyboardShortcut;
use muda::MenuItem;
use muda::Submenu;

impl BuiltMenu {
    pub(super) fn item(
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
                // AppKit owns these fonts; egui's atlas does not exist during menu installation.
                Some(keys.format(&eframe::egui::ModifierNames::SYMBOLS, context.os().is_mac()))
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

    pub(super) fn app(
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
}
