//! Distinguish first accepted-and-saved text from a focused frame at the final native font.
use eframe::App;
use eframe::egui::ColorImage;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::FontFamily;
use eframe::egui::Id;
use eframe::egui::RawInput;
use eframe::egui::Ui;
use eframe::egui::ViewportCommand;
use eframe::egui::ViewportId;
use eframe::egui::Visuals;
use quick_note::app::QuickNoteApp;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

const TEXT: &str = "startup 한💡";

pub(super) struct StartupProbe {
    pub app: QuickNoteApp,
    pub output: PathBuf,
    pub started: Instant,
    pub phases: Arc<Mutex<Value>>,
    pub input_reported: bool,
    pub screenshot_requested: bool,
    pub save_capture: bool,
    pub sender: mpsc::Sender<(Arc<ColorImage>, Instant, Value)>,
    pub receiver: mpsc::Receiver<(Arc<ColorImage>, Instant, Value)>,
}

pub(super) fn stamp(phases: &Mutex<Value>, name: &str, started: Instant) {
    phases
        .lock()
        .expect("only non-panicking timestamp assignments hold this mutex")[name] =
        json!(started.elapsed().as_secs_f64() * 1000.0);
}

impl App for StartupProbe {
    fn raw_input_hook(&mut self, _context: &Context, input: &mut RawInput) {
        // Never record the person's typing. Rebuild our one queued event across logic-only ticks.
        input
            .events
            .retain(|event| matches!(event, Event::WindowFocused(_)));
        if self.app.note.body.is_empty()
            && input.focused
            && input
                .viewports
                .get(&ViewportId::ROOT)
                .is_some_and(|viewport| viewport.focused == Some(true))
        {
            input.events.push(Event::Text(TEXT.into()));
        }
    }

    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        let context = ui.ctx();
        let now = self.started.elapsed().as_secs_f64() * 1000.0;
        let focused = context
            .input(|input| input.focused && input.viewport().focused == Some(true))
            && context
                .memory(|memory| memory.has_focus(Id::unique(("note-editor", &self.app.note.id))));
        let editor_font = context.fonts(|fonts| {
            fonts
                .definitions()
                .families
                .get(&FontFamily::Name("editor".into()))
                .and_then(|family| family.first())
                .cloned()
        });
        {
            let mut phases = self
                .phases
                .lock()
                .expect("timestamp assignments cannot poison this mutex");
            let phases = phases
                .as_object_mut()
                .expect("startup phases are an object");
            if !phases.contains_key("first_ui_ms") {
                eprintln!(
                    "first_ui: {}",
                    json!({"ms":now,"input_focused":context.input(|input|input.focused),
                        "viewport_focused":context.input(|input|input.viewport().focused),
                        "editor_focused":focused,"editor_font":editor_font,
                        "body":self.app.note.body})
                );
            }
            phases.entry("first_ui_ms").or_insert(json!(now));
            if focused {
                phases.entry("first_focused_ui_ms").or_insert(json!(now));
            }
            if editor_font.as_deref() == Some("platform-monospace") {
                if !phases.contains_key("native_font_ui_ms") {
                    eprintln!("native_font_ui: {now}");
                }
                phases.entry("native_font_ui_ms").or_insert(json!(now));
            }
        }
        if focused && !self.input_reported && self.app.note.body == TEXT && !self.app.dirty {
            let stored = self
                .app
                .store
                .note(&self.app.note.id)
                .expect("fixture database is readable")
                .expect("fixture note remains stored");
            assert_eq!(
                stored.body, TEXT,
                "the first text must be committed, not only painted"
            );
            assert_eq!(stored.selection.location, TEXT.encode_utf16().count());
            self.input_reported = true;
            stamp(&self.phases, "input_saved_ms", self.started);
            let report = json!({"phases":*self.phases.lock().expect("timestamp mutex is valid"),
                "editor_font":editor_font,"body":stored.body,"selection":stored.selection,
                "tabs":self.app.tabs.len(),"window_focused":true,"editor_focused":true});
            fs::write(
                self.output.join("input-ready.json"),
                serde_json::to_vec(&report).expect("report is serializable"),
            )
            .expect("measurement output is writable");
        }
        if self.input_reported
            && focused
            && !self.screenshot_requested
            && editor_font.as_deref() == Some("platform-monospace")
        {
            self.screenshot_requested = true;
            let sender = self.sender.clone();
            let repaint = context.clone();
            let report = json!({"phases":*self.phases.lock().expect("timestamp mutex is valid"),
                "editor_font":editor_font,"body":self.app.note.body,"tabs":self.app.tabs.len(),
                "window_focused":true,"editor_focused":true});
            context.request_screenshot(move |image| {
                let _ = sender.send((image, Instant::now(), report));
                repaint.request_repaint();
            });
        }
        if self.screenshot_requested {
            // Drive readback progress without waiting for the editor's caret blink.
            context.request_repaint_after(Duration::from_millis(1));
        }
        for (image, captured, mut report) in self.receiver.try_iter() {
            report["focused_frame_captured_ms"] =
                json!(captured.duration_since(self.started).as_secs_f64() * 1000.0);
            fs::write(
                self.output.join("captured.json"),
                serde_json::to_vec(&report).expect("report is serializable"),
            )
            .expect("measurement output is writable");
            if self.save_capture {
                let bytes: Vec<_> = image
                    .pixels
                    .iter()
                    .flat_map(|pixel| pixel.to_array())
                    .collect();
                image::save_buffer(
                    self.output.join("first-input.png"),
                    &bytes,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .expect("fixture screenshot is valid");
            }
            context.send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        self.app.clear_color(visuals)
    }
}
