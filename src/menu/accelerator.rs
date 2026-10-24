//! Native equivalents use logical keys and the same resolved bindings as the editor.
use eframe::egui::Key;
use eframe::egui::KeyboardShortcut;
use muda::accelerator::Key as NativeKey;
use muda::accelerator::KeyAccelerator;
use muda::accelerator::Modifiers;
use muda::accelerator::NamedKey;

pub(super) fn native(keys: KeyboardShortcut) -> Option<KeyAccelerator> {
    let key = match keys.logical_key {
        Key::ArrowDown => NativeKey::Named(NamedKey::ArrowDown),
        Key::ArrowLeft => NativeKey::Named(NamedKey::ArrowLeft),
        Key::ArrowRight => NativeKey::Named(NamedKey::ArrowRight),
        Key::ArrowUp => NativeKey::Named(NamedKey::ArrowUp),
        Key::Space => NativeKey::Character(" ".into()),
        Key::Minus => NativeKey::Character("-".into()),
        Key::Quote => NativeKey::Character("'".into()),
        key if key.symbol_or_name().chars().count() == 1 => {
            // AppKit matches shifted letters by case; the Shift flag alone is insufficient.
            NativeKey::Character(if keys.modifiers.shift {
                key.symbol_or_name().to_uppercase()
            } else {
                key.symbol_or_name().to_lowercase()
            })
        }
        key => NativeKey::Named(key.name().parse().ok()?),
    };
    let mut modifiers = Modifiers::empty();
    if keys.modifiers.command || keys.modifiers.mac_cmd {
        modifiers |= Modifiers::META;
    }
    if keys.modifiers.ctrl {
        modifiers |= Modifiers::CONTROL;
    }
    if keys.modifiers.shift {
        modifiers |= Modifiers::SHIFT;
    }
    if keys.modifiers.alt {
        modifiers |= Modifiers::ALT;
    }
    Some(KeyAccelerator::new(modifiers, key))
}
