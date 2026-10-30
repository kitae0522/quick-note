//! OS hotkeys wake the event loop; there is no background polling thread.
use crate::shortcut::Command;
use crate::storage::Result;
use eframe::egui::Context;
use eframe::egui::KeyboardShortcut;
use eframe::egui::ViewportCommand;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::GlobalHotKeyManager;
use global_hotkey::HotKeyState;
use global_hotkey::hotkey::HotKey;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

pub(crate) struct GlobalCapture {
    manager: GlobalHotKeyManager,
    keys: Vec<HotKey>,
    ids: Arc<Mutex<Vec<(u32, Command)>>>,
    receiver: mpsc::Receiver<(Command, bool)>,
}

impl GlobalCapture {
    pub fn new(context: &Context) -> Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let ids = Arc::new(Mutex::new(Vec::new()));
        let capture = Self {
            manager: GlobalHotKeyManager::new()?,
            keys: Vec::new(),
            ids: ids.clone(),
            receiver,
        };
        let context = context.clone();
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state != HotKeyState::Pressed {
                return;
            }
            let command = ids
                .lock()
                .expect("hotkey IDs lock")
                .iter()
                .find(|(id, _)| *id == event.id)
                .map(|(_, command)| *command);
            let Some(command) = command else {
                return;
            };
            let active = active(&context);
            if sender.send((command, active)).is_ok() {
                if !active && command != Command::CaptureClipboard {
                    #[cfg(target_os = "macos")]
                    if let Some(main) = objc2::MainThreadMarker::new() {
                        let app = objc2_app_kit::NSApplication::sharedApplication(main);
                        app.unhide(None);
                        app.activate();
                    }
                    context.send_viewport_cmd(ViewportCommand::Visible(true));
                    context.send_viewport_cmd(ViewportCommand::Minimized(false));
                    context.send_viewport_cmd(ViewportCommand::Focus);
                }
                context.request_repaint();
            }
        }));
        Ok(capture)
    }

    /// Register additions before releasing old shortcuts; a conflict retains the working set.
    pub fn update(&mut self, bindings: Vec<(Command, KeyboardShortcut)>) -> Result<()> {
        let bindings = bindings
            .into_iter()
            .map(|(command, keys)| hotkey(keys).map(|key| (command, key)))
            .collect::<Result<Vec<_>>>()?;
        let keys: Vec<_> = bindings.iter().map(|(_, key)| *key).collect();
        let additions: Vec<_> = keys
            .iter()
            .filter(|key| !self.keys.contains(key))
            .copied()
            .collect();
        for (index, key) in additions.iter().enumerate() {
            if let Err(error) = self.manager.register(*key) {
                for registered in &additions[..index] {
                    self.manager.unregister(*registered)?;
                }
                return Err(error.into());
            }
        }
        for key in self.keys.iter().filter(|key| !keys.contains(key)) {
            self.manager.unregister(*key)?;
        }
        *self.ids.lock().expect("hotkey IDs lock") = bindings
            .iter()
            .map(|(command, key)| (key.id(), *command))
            .collect();
        self.keys = keys;
        Ok(())
    }

    pub fn registered(&self) -> bool {
        !self.keys.is_empty()
    }

    pub fn receive(&self) -> Vec<(Command, bool)> {
        self.receiver.try_iter().collect()
    }
}

fn active(context: &Context) -> bool {
    #[cfg(target_os = "macos")]
    if let Some(main) = objc2::MainThreadMarker::new() {
        return objc2_app_kit::NSApplication::sharedApplication(main).isActive();
    }
    context.input(|input| input.focused)
}

fn hotkey(keys: KeyboardShortcut) -> Result<HotKey> {
    let mut parts = Vec::new();
    if keys.modifiers.ctrl || (keys.modifiers.command && !cfg!(target_os = "macos")) {
        parts.push("Ctrl");
    }
    if keys.modifiers.mac_cmd || (keys.modifiers.command && cfg!(target_os = "macos")) {
        parts.push("Super");
    }
    if keys.modifiers.alt {
        parts.push("Alt");
    }
    if keys.modifiers.shift {
        parts.push("Shift");
    }
    parts.push(keys.logical_key.name());
    Ok(parts.join("+").parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Key;
    use eframe::egui::Modifiers;

    #[test]
    fn native_shortcuts_preserve_platform_modifiers() -> Result<()> {
        assert_eq!(
            hotkey(KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::ALT,
                Key::N
            ))?,
            if cfg!(target_os = "macos") {
                "Super+Alt+N"
            } else {
                "Ctrl+Alt+N"
            }
            .parse()?
        );
        assert_eq!(
            hotkey(KeyboardShortcut::new(
                Modifiers::MAC_CMD | Modifiers::ALT,
                Key::N
            ))?,
            "Super+Alt+N".parse()?
        );
        assert_eq!(
            hotkey(KeyboardShortcut::new(
                Modifiers::CTRL | Modifiers::ALT,
                Key::N
            ))?,
            "Ctrl+Alt+N".parse()?
        );
        Ok(())
    }
}
