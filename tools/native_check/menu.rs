//! Observe and activate the actual AppKit menu, without cross-process Accessibility access.
use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use objc2_app_kit::NSEvent;
use objc2_app_kit::NSEventModifierFlags;
use objc2_app_kit::NSEventType;
use objc2_foundation::NSPoint;
use objc2_foundation::NSString;
use serde_json::Value;
use serde_json::json;

pub(super) fn snapshot() -> Value {
    let Some(main) = MainThreadMarker::new() else {
        return Value::Null;
    };
    let Some(menu) = NSApplication::sharedApplication(main).mainMenu() else {
        return Value::Null;
    };
    let menus: Vec<Value> =
        menu.itemArray()
            .iter()
            .map(|item| {
                let children =
                    item.submenu().map(|submenu| {
                        submenu.itemArray().iter().map(|child|
            json!({"title":child.title().to_string(),"enabled":child.isEnabled(),
                "key":child.keyEquivalent().to_string(),
                "modifiers":child.keyEquivalentModifierMask().bits()})).collect::<Vec<Value>>()
                    });
                json!({"title":item.title().to_string(),"items":children})
            })
            .collect();
    json!(menus)
}

pub(super) fn activate(menu_name: &str, item_name: &str) -> Result<(), String> {
    let main = MainThreadMarker::new().ok_or("menu activation requires the main thread")?;
    let menu = NSApplication::sharedApplication(main)
        .mainMenu()
        .ok_or("no system menu")?;
    let submenu = menu
        .itemArray()
        .iter()
        .find(|item| item.title().to_string() == menu_name)
        .and_then(|item| item.submenu())
        .ok_or_else(|| format!("missing menu: {menu_name}"))?;
    let index = submenu
        .itemArray()
        .iter()
        .position(|item| item.title().to_string() == item_name)
        .ok_or_else(|| format!("missing menu item: {item_name}"))?;
    if !submenu
        .itemAtIndex(index as isize)
        .is_some_and(|item| item.isEnabled())
    {
        return Err(format!("disabled menu item: {item_name}"));
    }
    submenu.performActionForItemAtIndex(index as isize);
    Ok(())
}

pub(super) fn key_equivalent(key: &str, shift: bool) -> Result<bool, String> {
    let main = MainThreadMarker::new().ok_or("menu activation requires the main thread")?;
    let menu = NSApplication::sharedApplication(main)
        .mainMenu()
        .ok_or("no system menu")?;
    let mut modifiers = NSEventModifierFlags::Command;
    if shift {
        modifiers |= NSEventModifierFlags::Shift;
    }
    let text = NSString::from_str(&if shift {
        key.to_uppercase()
    } else {
        key.to_owned()
    });
    let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
        NSEventType::KeyDown, NSPoint::new(0.0, 0.0), modifiers, 0.0, 0, None,
        &text, &text, false, 0).ok_or("could not construct a native menu key event")?;
    Ok(menu.performKeyEquivalent(&event))
}
