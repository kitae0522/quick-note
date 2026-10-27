use eframe::egui::Event;
use eframe::egui::InputState;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use quick_note::shortcut::Command;
use quick_note::shortcut::Platform;
use quick_note::shortcut::Shortcuts;
use quick_note::storage::Result;

fn press(shortcuts: &Shortcuts, key: Key, modifiers: Modifiers) -> Option<Command> {
    let mut input = InputState::default();
    input.events.push(Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    });
    shortcuts.consume(&mut input)
}

#[test]
fn find_defaults_and_the_retired_search_alias_work_on_both_platforms() -> Result<()> {
    for platform in [Platform::Mac, Platform::Windows] {
        for shortcuts in [
            Shortcuts::defaults(platform),
            Shortcuts::from_bytes(
                br#"{"schemaVersion":1,"bindings":{"search_notes":["Primary+P","Primary+F"]}}"#,
                platform,
            )?,
        ] {
            assert_eq!(
                press(&shortcuts, Key::F, Modifiers::COMMAND),
                Some(Command::Find)
            );
            assert_eq!(
                press(&shortcuts, Key::P, Modifiers::COMMAND),
                Some(Command::Search)
            );
            assert_eq!(
                press(&shortcuts, Key::F, Modifiers::COMMAND | Modifiers::ALT),
                Some(Command::Replace)
            );
            assert_eq!(
                press(&shortcuts, Key::G, Modifiers::COMMAND),
                Some(Command::FindNext)
            );
        }
    }
    Ok(())
}

#[test]
fn new_defaults_preserve_custom_keys_and_never_rewrite_the_configuration() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let folder = directory.path().join("shortcut");
    std::fs::create_dir(&folder)?;
    let path = folder.join("bindings.json");
    let custom = br#"{"schemaVersion":1,"bindings":{"search_notes":["Primary+F"],"save_note":["Primary+G"]}}"#;
    std::fs::write(&path, custom)?;
    let shortcuts = Shortcuts::load(directory.path(), Platform::Mac)?;
    assert_eq!(
        press(&shortcuts, Key::F, Modifiers::COMMAND),
        Some(Command::Search)
    );
    assert_eq!(
        press(&shortcuts, Key::G, Modifiers::COMMAND),
        Some(Command::Save)
    );
    assert_eq!(
        press(&shortcuts, Key::F3, Modifiers::NONE),
        Some(Command::FindNext)
    );
    assert_eq!(std::fs::read(path)?, custom);
    Ok(())
}
