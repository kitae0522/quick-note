//! Translate isolated diagnostic key commands into egui input batches.
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use serde_json::Value;

pub(super) fn events(command: &Value) -> Result<Vec<Event>, String> {
    let key = match command["key"].as_str().unwrap_or("") {
        "n" => Key::N,
        "k" => Key::K,
        "p" => Key::P,
        "f" => Key::F,
        "g" => Key::G,
        "f3" => Key::F3,
        "m" => Key::M,
        "w" => Key::W,
        "q" => Key::Q,
        "s" => Key::S,
        "z" => Key::Z,
        "a" => Key::A,
        "escape" => Key::Escape,
        "enter" => Key::Enter,
        "space" => Key::Space,
        "tab" => Key::Tab,
        "backspace" => Key::Backspace,
        "left" => Key::ArrowLeft,
        "right" => Key::ArrowRight,
        "up" => Key::ArrowUp,
        "down" => Key::ArrowDown,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "f6" => Key::F6,
        "slash" => Key::Slash,
        "comma" => Key::Comma,
        "1" => Key::Num1,
        "9" => Key::Num9,
        "l" => Key::L,
        other => {
            return Err(format!("Unknown key: {other}"));
        }
    };
    let mut events = Vec::new();
    let command_key = command["command"].as_bool().unwrap_or(false);
    let modifiers = Modifiers {
        command: command_key,
        mac_cmd: command_key && cfg!(target_os = "macos"),
        ctrl: (command_key && !cfg!(target_os = "macos"))
            || command["control"].as_bool().unwrap_or(false),
        shift: command["shift"].as_bool().unwrap_or(false),
        alt: command["alt"].as_bool().unwrap_or(false),
    };
    events.push(Event::ModifiersChanged(modifiers));
    for pressed in [true, false] {
        events.push(Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        });
    }
    events.push(Event::ModifiersChanged(Modifiers::NONE));
    Ok(events)
}
