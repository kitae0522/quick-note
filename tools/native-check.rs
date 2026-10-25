//! Isolated native-window diagnostics; production never imports this driver.
#[cfg(target_os = "macos")]
#[path = "native_check/appkit.rs"]
mod appkit;
#[path = "native_check/capture.rs"]
mod capture;
#[cfg(target_os = "macos")]
#[path = "native_check/clipboard.rs"]
mod clipboard;
#[path = "native_check/input.rs"]
mod input;
#[cfg(test)]
#[path = "native_check/input_tests.rs"]
mod input_tests;
#[cfg(target_os = "macos")]
#[path = "native_check/menu.rs"]
mod menu;
#[path = "native_check/probe.rs"]
mod probe;
#[path = "native_check/wake.rs"]
mod wake;
use quick_note::app::QuickNoteApp;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Instant;

use probe::Probe;

fn main() -> eframe::Result {
    let started = Instant::now();
    let mut args = std::env::args_os().skip(1);
    let mut directory = None;
    let mut output = None;
    let mut exit_after_first = false;
    while let Some(arg) = args.next() {
        if arg == "--data-directory" {
            directory = args.next().map(PathBuf::from);
        } else if arg == "--output" {
            output = args.next().map(PathBuf::from);
        } else if arg == "--startup-only" {
            exit_after_first = true;
        }
    }
    let directory = directory.expect("--data-directory must point to isolated test data");
    let output = output.expect("--output is required");
    fs::create_dir_all(&output).expect("isolated verification fixture should be valid");
    let mut app =
        QuickNoteApp::try_new(directory).expect("isolated verification fixture should be valid");
    let phases = Arc::new(Mutex::new(
        json!({"storage_ms":started.elapsed().as_secs_f64()*1000.0}),
    ));
    let (screenshots_tx, screenshots_rx) = mpsc::channel();
    let mut options = quick_note::native::options("Quick Note · Native verification");
    let event_phases = phases.clone();
    options.event_loop_builder = Some(Box::new(move |_| {
        event_phases
            .lock()
            .expect("diagnostic phase mutex is not poisoned")["event_loop_ms"] =
            json!(started.elapsed().as_secs_f64() * 1000.0);
    }));
    let window_phases = phases.clone();
    options.window_builder = Some(Box::new(move |builder| {
        window_phases
            .lock()
            .expect("diagnostic phase mutex is not poisoned")["window_builder_ms"] =
            json!(started.elapsed().as_secs_f64() * 1000.0);
        builder
    }));
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        let descriptor = setup.device_descriptor.clone();
        let device_phases = phases.clone();
        setup.device_descriptor = Arc::new(move |adapter| {
            device_phases
                .lock()
                .expect("diagnostic phase mutex is not poisoned")["device_request_ms"] =
                json!(started.elapsed().as_secs_f64() * 1000.0);
            descriptor(adapter)
        });
    }
    eframe::run_native(
        "Quick Note Native Verification",
        options,
        Box::new(move |creation| {
            phases
                .lock()
                .expect("diagnostic phase mutex is not poisoned")["gpu_ready_ms"] =
                json!(started.elapsed().as_secs_f64() * 1000.0);
            app.configure(&creation.egui_ctx);
            app.install_menu(&creation.egui_ctx);
            phases
                .lock()
                .expect("diagnostic phase mutex is not poisoned")["configured_ms"] =
                json!(started.elapsed().as_secs_f64() * 1000.0);
            Ok(Box::new(Probe {
                _input_wake: wake::InputWake::start(creation.egui_ctx.clone(), output.clone())?,
                queued_events: Vec::new(),
                app,
                output,
                started,
                requested_first: false,
                ready: false,
                first_paint_ms: None,
                exit_after_first,
                sequence: 0,
                pending: None,
                native_pending: None,
                native_expected: None,
                native_delivered: false,
                native_mode: false,
                pending_frames: HashMap::new(),
                screenshots_tx,
                screenshots_rx,
                phases,
                clipboard_requested: false,
                menu_key_handled: None,
                #[cfg(target_os = "macos")]
                clipboard_backup: None,
            }))
        }),
    )
}
