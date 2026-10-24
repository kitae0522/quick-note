//! Resource-only native input driver: no screenshot readback or command-file polling.
use eframe::App;
use eframe::Frame;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::RawInput;
use eframe::egui::Ui;
use eframe::egui::Visuals;
use quick_note::app;
use quick_note::app::Mode;
use quick_note::app::QuickNoteApp;
use quick_note::native;
use serde_json::json;
use std::collections::VecDeque;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Copy)]
enum Workload {
    Typing,
    Search,
    Markdown { enabled: bool, warm_preview: bool },
    Preview,
}

struct ResourceWorkload {
    app: QuickNoteApp,
    workload: Workload,
    next_input: Instant,
    inputs: usize,
    frames: usize,
    report: PathBuf,
    started: Instant,
    ui_ms: VecDeque<f64>,
}

impl App for ResourceWorkload {
    fn raw_input_hook(&mut self, context: &Context, input: &mut RawInput) {
        input.events.clear();
        if matches!(self.workload, Workload::Markdown { .. } | Workload::Preview) {
            return;
        }
        if Instant::now() < self.next_input {
            return;
        }
        input.events.push(Event::WindowFocused(true));
        self.next_input = Instant::now() + Duration::from_millis(125);
        self.inputs += 1;
        match self.workload {
            Workload::Typing => input.events.push(Event::Text("a".into())),
            Workload::Search => {
                const QUERIES: [&str; 8] = [
                    "i", "id", "ide", "idea", "idea 0", "idea 05", "idea 052", "marker",
                ];
                self.app.header.query = QUERIES[self.inputs % QUERIES.len()].into();
                self.app.request_search(context);
            }
            Workload::Markdown { .. } => {}
            Workload::Preview => {}
        }
    }

    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        self.frames += 1;
        if let Workload::Markdown {
            enabled,
            warm_preview,
        } = self.workload
        {
            if self.frames == 1 && warm_preview {
                self.app.mode = Mode::Preview;
            } else if self.frames == 3 || (self.frames == 1 && !warm_preview) {
                self.app
                    .set_markdown_enabled(enabled)
                    .expect("isolated resource fixture settings must be writable");
                self.app.mode = Mode::Edit;
            }
        }
        if matches!(self.workload, Workload::Preview) {
            self.app.mode = Mode::Preview;
        }
        let start = Instant::now();
        self.app.ui(ui, frame);
        self.ui_ms.push_back(start.elapsed().as_secs_f64() * 1000.0);
        if self.ui_ms.len() > 1024 {
            self.ui_ms.pop_front();
        }
        if self.frames <= 3 || self.frames.is_multiple_of(16) {
            let report = json!({
                "frames": self.frames,
                "ui_ms": self.ui_ms,
                "inputs": self.inputs,
                "elapsed_seconds": self.started.elapsed().as_secs_f64(),
                "body_bytes": self.app.note.body.len(),
                "dirty": self.app.dirty,
                "error": self.app.error.as_ref().map(|error| self.app.locale.error(error.as_ref())),
                "results": self.app.search_results().len(),
                "markdown_enabled": self.app.markdown_enabled(),
                "mode": if self.app.mode == Mode::Preview { "preview" } else { "edit" },
                "measurement": if matches!(self.workload, Workload::Markdown { .. }) {
                    "source editor at 8 Hz; physical input discarded; no GPU readback"
                } else { "production UI with synthetic 8 Hz input; no GPU readback" }
            });
            let _reported = app::atomic_write(&self.report, report.to_string().as_bytes());
        }
        ui.ctx().request_repaint_after(Duration::from_millis(125));
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        self.app.clear_color(visuals)
    }
}

fn main() -> eframe::Result {
    let mut arguments = env::args_os().skip(1);
    let directory = arguments
        .next()
        .map(PathBuf::from)
        .expect("give an isolated data directory");
    let report = arguments
        .next()
        .map(PathBuf::from)
        .expect("give a diagnostic report path");
    let workload = match arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .as_deref()
    {
        Some("typing") => Workload::Typing,
        Some("search") => Workload::Search,
        Some("preview") => Workload::Preview,
        Some("markdown-on") => Workload::Markdown {
            enabled: true,
            warm_preview: true,
        },
        Some("markdown-off") => Workload::Markdown {
            enabled: false,
            warm_preview: true,
        },
        Some("markdown-fresh-on") => Workload::Markdown {
            enabled: true,
            warm_preview: false,
        },
        Some("markdown-fresh-off") => Workload::Markdown {
            enabled: false,
            warm_preview: false,
        },
        _ => panic!("choose typing, search, markdown-on/off or markdown-fresh-on/off"),
    };
    let mut app = QuickNoteApp::try_new(directory).expect("the isolated fixture store should open");
    let mut options = native::options("Quick Note · Resource verification");
    options.viewport = options.viewport.with_active(false).with_always_on_top();
    eframe::run_native(
        "Quick Note resource workload",
        options,
        Box::new(move |creation| {
            app.configure(&creation.egui_ctx);
            app.install_menu(&creation.egui_ctx);
            Ok(Box::new(ResourceWorkload {
                app,
                workload,
                next_input: Instant::now(),
                inputs: 0,
                frames: 0,
                report,
                started: Instant::now(),
                ui_ms: VecDeque::new(),
            }))
        }),
    )
}
