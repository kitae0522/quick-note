//! Reproduce input hooks called repeatedly before a visible UI pass consumes input.
use super::probe::Probe;
use super::wake::InputWake;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::RawInput;
use egui_kittest::Harness;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Instant;

fn replay_twice(command: Value, clipboard: Option<&str>) -> Result<String> {
    let directory = tempfile::tempdir()?;
    let output = directory.path().join("probe");
    fs::create_dir(&output)?;
    fs::write(output.join("command.json"), serde_json::to_vec(&command)?)?;
    let context = Context::default();
    let (screenshots_tx, screenshots_rx) = mpsc::channel();
    let mut probe = Probe {
        _input_wake: InputWake::start(context.clone(), output.clone())?,
        queued_events: Vec::new(),
        app: QuickNoteApp::try_new(directory.path().join("notes"))?,
        output,
        started: Instant::now(),
        requested_first: false,
        ready: true,
        first_paint_ms: None,
        exit_after_first: false,
        sequence: 0,
        pending: None,
        pending_frames: HashMap::new(),
        screenshots_tx,
        screenshots_rx,
        phases: Arc::new(Mutex::new(Value::Null)),
        clipboard_requested: clipboard.is_some(),
        menu_key_handled: None,
        #[cfg(target_os = "macos")]
        clipboard_backup: None,
    };
    let mut first = RawInput::default();
    first
        .events
        .push(Event::Text("UNRELATED PHYSICAL INPUT".into()));
    if let Some(clipboard) = clipboard {
        first.events.push(Event::Paste(clipboard.into()));
    }
    probe.feed_input(&context, &mut first);
    let mut retry = first.clone();
    probe.feed_input(&context, &mut retry);
    assert_eq!(probe.app.note.body, "");
    let Probe { mut app, .. } = probe;
    let mut harness = Harness::builder()
        .with_size([640.0, 400.0])
        .build_eframe(move |creation| {
            app.configure(&creation.egui_ctx);
            app
        });
    harness.run();
    harness.input_mut().events = retry.events;
    harness.run();
    Ok(harness.state().note.body.clone())
}

#[test]
fn a_text_command_survives_logic_only_ticks_and_edits_exactly_once() -> Result<()> {
    let body = replay_twice(json!({"sequence":1,"type":"text","text":"("}), None)?;
    assert_eq!(body, "()");
    Ok(())
}

#[test]
fn an_expected_clipboard_event_survives_logic_only_ticks_without_pairing() -> Result<()> {
    let body = replay_twice(json!({"sequence":1,"type":"report"}), Some("("))?;
    assert_eq!(body, "(");
    Ok(())
}
