//! Inspect AppKit's actual hit testing before egui receives pointer events.
use eframe::egui::Event;
use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use objc2_app_kit::NSEvent;
use objc2_app_kit::NSEventModifierFlags;
use objc2_app_kit::NSEventType;
use objc2_app_kit::NSWindowButton;
use objc2_foundation::NSPoint;
use objc2_foundation::NSString;
use quick_note::storage::Result;
use serde_json::Value;
use serde_json::json;

pub(super) fn post(command: &Value) -> Result<()> {
    let main = MainThreadMarker::new().expect("native input is posted on the AppKit thread");
    let app = NSApplication::sharedApplication(main);
    let window = app.keyWindow().ok_or("Native fixture needs a key window")?;
    if command["type"] == "native_pointer" {
        let kind = match command["pressed"].as_bool() {
            Some(true) => NSEventType::LeftMouseDown,
            Some(false) => NSEventType::LeftMouseUp,
            None if command["dragged"].as_bool() == Some(true) => NSEventType::LeftMouseDragged,
            None => NSEventType::MouseMoved,
        };
        let height = window
            .contentView()
            .ok_or("Native content view is missing")?
            .bounds()
            .size
            .height;
        let event = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            kind, NSPoint::new(command["x"].as_f64().unwrap_or(0.0), height - command["y"].as_f64().unwrap_or(0.0)),
            NSEventModifierFlags::empty(), 0.0, window.windowNumber(), None, 0, 1, 1.0,
        ).ok_or("Could not create native pointer event")?;
        app.postEvent_atStart(&event, false);
    } else {
        let text = NSString::from_str("7");
        for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
            let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
                kind, NSPoint::new(0.0, 0.0), NSEventModifierFlags::empty(), 0.0,
                window.windowNumber(), None, &text, &text, false, 26,
            ).ok_or("Could not create native key event")?;
            app.postEvent_atStart(&event, false);
        }
    }
    Ok(())
}

pub(super) fn matches(command: &Value, event: &Event) -> bool {
    if command["type"] == "native_key" {
        return matches!(event, Event::Text(text) if text == "7");
    }
    let expected = eframe::egui::pos2(
        command["x"].as_f64().unwrap_or(0.0) as f32,
        command["y"].as_f64().unwrap_or(0.0) as f32,
    );
    match event {
        Event::PointerMoved(pos) => command["pressed"].is_null() && pos.distance(expected) < 1.0,
        Event::PointerButton { pos, pressed, .. } => {
            command["pressed"].as_bool() == Some(*pressed) && pos.distance(expected) < 1.0
        }
        _ => false,
    }
}

pub(super) fn snapshot() -> Option<Value> {
    let main = MainThreadMarker::new()?;
    let window = NSApplication::sharedApplication(main).keyWindow()?;
    let content = window.contentView()?;
    let close = window.standardWindowButton(NSWindowButton::CloseButton)?;
    let root = content.ancestorSharedWithView(&close)?;
    let height = content.bounds().size.height;
    let hits: Vec<_> = [(450.0, 22.0), (450.0, 100.0), (450.0, 400.0)]
        .into_iter()
        .map(|(x, y)| {
            let point = NSPoint::new(x, if content.isFlipped() { y } else { height - y });
            let point = root.convertPoint_fromView(point, Some(&content));
            json!({"x":x,"y":y,"view":root.hitTest(point).map(|view|view.class().name().to_string_lossy().into_owned())})
        })
        .collect();
    Some(json!({
        "window_frame":[window.frame().origin.x,window.frame().origin.y,window.frame().size.width,window.frame().size.height],
        "first_responder":window.firstResponder().map(|view|view.class().name().to_string_lossy().into_owned()),
        "content_class":content.class().name().to_string_lossy().into_owned(),
        "content_can_move_window":content.mouseDownCanMoveWindow(),
        "content_frame":format!("{:?}",content.frame()),
        "root_class":root.class().name().to_string_lossy().into_owned(),
        "root_frame":format!("{:?}",root.frame()),
        "hit_tests":hits
    }))
}
