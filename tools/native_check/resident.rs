//! Drive the actual NSStatusItem menu while hidden, without focus helpers.
use super::probe::Probe;
use eframe::egui::Context;
use objc2_app_kit::NSMenu;
use serde_json::Value;
use serde_json::json;
use std::time::Instant;

impl Probe {
    pub(super) fn feed_resident(&mut self, context: &Context, command: &Value) -> bool {
        let kind = command["type"].as_str().unwrap_or_default();
        if ![
            "resident",
            "resident_report",
            "clipboard_fixture",
            "clipboard_restore",
        ]
        .contains(&kind)
        {
            return false;
        }
        let Some(sequence) = command["sequence"]
            .as_u64()
            .filter(|seq| *seq > self.sequence)
        else {
            return true;
        };
        self.sequence = sequence;
        self.resident_pending = Some((sequence, Instant::now()));
        let result = match kind {
            "resident" => self
                .app
                .resident_menu()
                .ok_or("no resident menu".to_owned())
                .and_then(|menu| activate(&menu, command["item"].as_str().unwrap_or_default())),
            "clipboard_fixture" => {
                self.set_clipboard_fixture(command["text"].as_str().unwrap_or_default())
            }
            "clipboard_restore" => self.restore_clipboard(),
            _ => Ok(()),
        };
        if let Err(error) = result {
            self.fail(error);
        }
        context.request_repaint();
        true
    }

    pub(super) fn restore_clipboard(&mut self) -> Result<(), String> {
        if let Some(backup) = &mut self.clipboard_backup {
            backup.restore()?;
        }
        self.clipboard_backup = None;
        Ok(())
    }

    fn set_clipboard_fixture(&mut self, text: &str) -> Result<(), String> {
        if self.clipboard_backup.is_none() {
            self.clipboard_backup = Some(super::clipboard::ClipboardBackup::capture()?);
        }
        arboard::Clipboard::new()
            .and_then(|mut board| board.set_text(text))
            .map_err(|error| error.to_string())
    }

    pub(super) fn resident_snapshot(&self) -> Value {
        self.app
            .resident_menu()
            .map_or(Value::Null, |menu| snapshot(&menu))
    }
}

fn activate(menu: &NSMenu, title: &str) -> Result<(), String> {
    for (index, item) in menu.itemArray().iter().enumerate() {
        if item.title().to_string() == title && item.isEnabled() && item.submenu().is_none() {
            menu.performActionForItemAtIndex(index as isize);
            return Ok(());
        }
        if let Some(submenu) = item.submenu()
            && activate(&submenu, title).is_ok()
        {
            return Ok(());
        }
    }
    Err(format!("missing resident menu item: {title}"))
}

fn snapshot(menu: &NSMenu) -> Value {
    Value::Array(
        menu.itemArray()
            .iter()
            .map(|item| {
                json!({
                    "title": item.title().to_string(), "enabled": item.isEnabled(),
                    "items": item.submenu().map(|menu| snapshot(&menu)),
                })
            })
            .collect(),
    )
}
