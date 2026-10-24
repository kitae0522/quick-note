//! One resolved binding set feeds dispatch and every displayed shortcut hint.
use super::ACTIONS;
use super::Command;
use super::Platform;
use super::key;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::InputState;
use eframe::egui::KeyboardShortcut;

pub(super) struct Binding {
    pub keys: KeyboardShortcut,
    pub action: usize,
}

pub struct Shortcuts {
    pub(super) bindings: Vec<Binding>,
    pub(super) platform: Platform,
}

impl Shortcuts {
    pub(crate) fn keys(&self, command: Command) -> impl Iterator<Item = KeyboardShortcut> + '_ {
        self.bindings
            .iter()
            .filter(move |binding| ACTIONS[binding.action].command == command)
            .map(|binding| binding.keys)
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn assigned(&self, keys: KeyboardShortcut) -> bool {
        self.bindings.iter().any(|binding| {
            key::fingerprint(
                binding.keys.logical_key,
                binding.keys.modifiers,
                self.platform,
            ) == key::fingerprint(keys.logical_key, keys.modifiers, self.platform)
        })
    }
    pub fn consume(&self, input: &mut InputState) -> Option<Command> {
        let (index, action) = input.events.iter().enumerate().find_map(|(index, event)| {
            let Event::Key {
                key,
                modifiers,
                pressed: true,
                ..
            } = event
            else {
                return None;
            };
            let pressed = key::fingerprint(*key, *modifiers, self.platform);
            self.bindings
                .iter()
                .find(|binding| {
                    key::fingerprint(
                        binding.keys.logical_key,
                        binding.keys.modifiers,
                        self.platform,
                    ) == pressed
                })
                .map(|binding| (index, binding.action))
        })?;
        input.events.remove(index);
        Some(ACTIONS[action].command)
    }

    #[must_use]
    pub fn key_text(&self, command: Command, context: &Context) -> String {
        self.keys(command)
            .map(|keys| context.format_shortcut(&keys))
            .collect::<Vec<_>>()
            .join(" / ")
    }
}
