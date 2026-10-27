//! Parse portable key names and compare physical modifiers exactly, including Shift and Alt.
use crate::i18n;
use crate::storage::Result;
use eframe::egui::Key;
use eframe::egui::KeyboardShortcut;
use eframe::egui::Modifiers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    Mac,
    Windows,
}

impl Platform {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else {
            Self::Windows
        }
    }
}

pub(super) fn fingerprint(key: Key, modifiers: Modifiers, platform: Platform) -> (Key, u8) {
    let control = modifiers.ctrl || (modifiers.command && platform == Platform::Windows);
    let command = modifiers.mac_cmd || (modifiers.command && platform == Platform::Mac);
    (
        key,
        u8::from(control)
            | (u8::from(command) << 1)
            | (u8::from(modifiers.alt) << 2)
            | (u8::from(modifiers.shift) << 3),
    )
}

pub(super) fn parse(source: &str, platform: Platform) -> Result<KeyboardShortcut> {
    let invalid = || i18n::message(|m| &m.error_shortcut_key).arg("shortcut", source);
    let mut parts: Vec<_> = source.split('+').map(str::trim).collect();
    let name = parts.pop().ok_or_else(invalid)?;
    let key = Key::from_name(name).ok_or_else(invalid)?;
    let mut modifiers = Modifiers::NONE;
    for part in parts {
        let before = fingerprint(key, modifiers, platform);
        match part.to_ascii_lowercase().as_str() {
            "primary" | "cmdorctrl" => modifiers.command = true,
            "ctrl" | "control" => modifiers.ctrl = true,
            "cmd" | "command" if platform == Platform::Mac => modifiers.mac_cmd = true,
            "alt" | "option" => modifiers.alt = true,
            "shift" => modifiers.shift = true,
            _ => return Err(invalid().into()),
        }
        if before == fingerprint(key, modifiers, platform) {
            return Err(invalid().into());
        }
    }
    let function_key = key
        .name()
        .strip_prefix('F')
        .and_then(|name| name.parse::<u8>().ok())
        .is_some_and(|number| (1..=35).contains(&number));
    if !(function_key || modifiers.command || modifiers.ctrl || modifiers.mac_cmd || modifiers.alt)
    {
        return Err(i18n::message(|m| &m.error_shortcut_reserved)
            .arg("shortcut", source)
            .into());
    }
    Ok(KeyboardShortcut::new(modifiers, key))
}
