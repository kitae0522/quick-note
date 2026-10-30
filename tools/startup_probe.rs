//! Measure normal app startup without the interaction driver's focus or polling helpers.
#[path = "startup/probe.rs"]
mod probe;

use probe::StartupProbe;
use quick_note::app::QuickNoteApp;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Instant;

fn main() -> eframe::Result {
    let started = Instant::now();
    let mut arguments = std::env::args_os().skip(1);
    let mut directory = None;
    let mut output = None;
    let mut capture = false;
    while let Some(argument) = arguments.next() {
        if argument == "--data-directory" {
            directory = arguments.next().map(PathBuf::from);
        } else if argument == "--output" {
            output = arguments.next().map(PathBuf::from);
        } else if argument == "--capture" {
            capture = true;
        }
    }
    let output = output.expect("--output must identify an isolated measurement directory");
    fs::create_dir_all(&output).expect("measurement output must be writable");
    let mut app = QuickNoteApp::try_new(directory.expect("--data-directory is required"))
        .expect("startup probe requires valid isolated fixture data");
    assert!(
        app.note.body.is_empty(),
        "startup probe requires an empty fixture draft"
    );
    let phases = Arc::new(Mutex::new(json!({})));
    probe::stamp(&phases, "app_state_ms", started);
    let mut options = quick_note::native::options(&app.locale.text().app_name);
    let event_phases = phases.clone();
    options.event_loop_builder = Some(Box::new(move |_| {
        probe::stamp(&event_phases, "event_loop_ms", started);
    }));
    let window_phases = phases.clone();
    options.window_builder = Some(Box::new(move |builder| {
        probe::stamp(&window_phases, "window_builder_ms", started);
        builder
    }));
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        let descriptor = setup.device_descriptor.clone();
        let device_phases = phases.clone();
        setup.device_descriptor = Arc::new(move |adapter| {
            probe::stamp(&device_phases, "device_request_ms", started);
            descriptor(adapter)
        });
    }
    let title = app.locale.text().app_name.clone();
    eframe::run_native(
        &title,
        options,
        Box::new(move |creation| {
            probe::stamp(&phases, "gpu_ready_ms", started);
            app.configure(&creation.egui_ctx);
            app.install_menu(&creation.egui_ctx);
            app.install_capture(&creation.egui_ctx);
            app.install_resident(&creation.egui_ctx);
            probe::stamp(&phases, "configured_ms", started);
            let (sender, receiver) = mpsc::channel();
            Ok(Box::new(StartupProbe {
                app,
                output,
                started,
                phases,
                input_reported: false,
                screenshot_requested: false,
                save_capture: capture,
                sender,
                receiver,
            }))
        }),
    )
}
