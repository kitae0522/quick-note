use eframe::egui::Event;
use eframe::egui::InputState;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use quick_note::shortcut::Command;
use quick_note::shortcut::Platform;
use quick_note::shortcut::Shortcuts;
use quick_note::storage::Result;
use std::fs;

fn press(shortcuts: &Shortcuts, key: Key, modifiers: Modifiers) -> Option<Command> {
    let mut input = InputState::default();
    input.events.push(Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    });
    let command = shortcuts.consume(&mut input);
    assert_eq!(input.events.is_empty(), command.is_some());
    command
}

#[test]
fn portable_defaults_and_overrides_use_exact_modifiers_aliases_and_disabled_actions() -> Result<()>
{
    for platform in [Platform::Mac, Platform::Windows] {
        let defaults = Shortcuts::defaults(platform);
        assert_eq!(
            press(&defaults, Key::N, Modifiers::COMMAND),
            Some(Command::New)
        );
        assert_eq!(
            press(&defaults, Key::W, Modifiers::COMMAND),
            Some(Command::CloseTab)
        );
        assert_eq!(
            press(&defaults, Key::W, Modifiers::COMMAND | Modifiers::SHIFT),
            Some(Command::Close)
        );
        assert_eq!(
            press(&defaults, Key::Tab, Modifiers::CTRL | Modifiers::SHIFT),
            Some(Command::PreviousNote)
        );
        assert_eq!(press(&defaults, Key::Tab, Modifiers::NONE), None);
        assert_eq!(
            press(&defaults, Key::N, Modifiers::COMMAND | Modifiers::SHIFT),
            None
        );
        let custom = Shortcuts::from_bytes(br#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Shift+N","Alt+N"],"save_note":[]}}"#, platform)?;
        assert_eq!(press(&custom, Key::N, Modifiers::COMMAND), None);
        assert_eq!(
            press(&custom, Key::N, Modifiers::COMMAND | Modifiers::SHIFT),
            Some(Command::New)
        );
        assert_eq!(press(&custom, Key::N, Modifiers::ALT), Some(Command::New));
        assert_eq!(
            press(
                &custom,
                Key::N,
                Modifiers::COMMAND | Modifiers::SHIFT | Modifiers::ALT
            ),
            Some(Command::NewCapture)
        );
        assert_eq!(press(&custom, Key::S, Modifiers::COMMAND), None);
        assert_eq!(
            press(&custom, Key::W, Modifiers::COMMAND),
            Some(Command::CloseTab)
        );
    }
    Ok(())
}

#[test]
fn invalid_keys_actions_conflicts_and_documents_are_rejected_before_dispatch() {
    for source in [
        r#"{"schemaVersion":2,"bindings":{}}"#,
        r#"{"schemaVersion":1,"bindings":{"not_an_action":[]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+W"],"close_tab":["Primary+W"]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+N","Primary+N"]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Primary+N"]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Super+NoSuchKey"]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":["N"]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Tab"]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":["Shift+Tab"]}}"#,
        r#"{"schemaVersion":1,"bindings":{"new_note":4}}"#,
        r#"{"schemaVersion":1,"bindings":{},"typo":true}"#,
        "not JSON",
    ] {
        for platform in [Platform::Mac, Platform::Windows] {
            assert!(
                Shortcuts::from_bytes(source.as_bytes(), platform).is_err(),
                "{platform:?}: {source}"
            );
        }
    }
    assert!(Shortcuts::from_bytes(&[0xff, 0xfe], Platform::Mac).is_err());
    assert!(Shortcuts::from_bytes(&vec![b' '; 65537], Platform::Mac).is_err());
    let too_many = serde_json::json!({"schemaVersion":1,"bindings":{"new_note":["F1","F2","F3","F4","F5","F7","F8","F9","F10"]}});
    assert!(
        Shortcuts::from_bytes(
            &serde_json::to_vec(&too_many).expect("JSON fixture"),
            Platform::Mac
        )
        .is_err()
    );
}

#[test]
fn physical_modifier_aliases_conflict_on_the_platform_where_they_are_identical() -> Result<()> {
    let source = br#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Shift+N"],"save_note":["Ctrl+Shift+N"]}}"#;
    let mac = Shortcuts::from_bytes(source, Platform::Mac)?;
    assert_eq!(
        press(&mac, Key::N, Modifiers::CTRL | Modifiers::SHIFT),
        Some(Command::Save)
    );
    assert_eq!(
        press(&mac, Key::N, Modifiers::MAC_CMD | Modifiers::SHIFT),
        Some(Command::New)
    );
    assert!(Shortcuts::from_bytes(source, Platform::Windows).is_err());
    let duplicate = br#"{"schemaVersion":1,"bindings":{"new_note":["Primary+N","Cmd+N"]}}"#;
    assert!(Shortcuts::from_bytes(duplicate, Platform::Mac).is_err());
    let directory = tempfile::tempdir()?;
    Shortcuts::load(directory.path(), Platform::Mac)?;
    let path = directory.path().join("shortcut/bindings.json");
    let invalid = b"{invalid configuration";
    fs::write(&path, invalid)?;
    assert!(Shortcuts::load(directory.path(), Platform::Mac).is_err());
    assert_eq!(fs::read(path)?, invalid);
    Ok(())
}

#[test]
fn old_defaults_migrate_without_rewriting_custom_shortcut_files() -> Result<()> {
    let directory = tempfile::tempdir()?;
    fs::create_dir(directory.path().join("shortcut"))?;
    let path = directory.path().join("shortcut/bindings.json");
    let original = r#"{"schemaVersion":1,"bindings":{"delete_note":["Primary+W"],"close_window":["Primary+Shift+W","Primary+Q"]}}"#;
    fs::write(&path, original)?;
    let shortcuts = Shortcuts::load(directory.path(), Platform::Mac)?;
    assert_eq!(
        press(&shortcuts, Key::W, Modifiers::COMMAND),
        Some(Command::CloseTab)
    );
    assert_eq!(
        press(&shortcuts, Key::Q, Modifiers::COMMAND),
        Some(Command::Quit)
    );
    assert_eq!(
        press(
            &shortcuts,
            Key::Backspace,
            Modifiers::COMMAND | Modifiers::SHIFT
        ),
        Some(Command::Delete)
    );
    assert_eq!(fs::read_to_string(&path)?, original);
    let custom = Shortcuts::from_bytes(br#"{"schemaVersion":1,"bindings":{"new_note":["Primary+Alt+N"],"delete_note":["Primary+D"]}}"#, Platform::Mac)?;
    assert_eq!(
        press(&custom, Key::N, Modifiers::COMMAND | Modifiers::ALT),
        Some(Command::New)
    );
    assert_eq!(
        press(&custom, Key::D, Modifiers::COMMAND),
        Some(Command::Delete)
    );
    Ok(())
}
