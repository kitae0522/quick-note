//! Isolated native-window driver. Production never imports this fixture harness.
use eframe::egui::ColorImage;
use eframe::egui::Context;
use eframe::egui::DragAndDrop;
use eframe::egui::Event;
use eframe::egui::RawInput;
use eframe::egui::Ui;
use eframe::egui::ViewportCommand;
use eframe::egui::Visuals;
use quick_note::app;
use quick_note::app::Mode;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Instant;

pub(super) struct Probe {
    pub(super) _input_wake: super::wake::InputWake,
    pub(super) queued_events: Vec<Event>,
    pub(super) app: QuickNoteApp,
    pub(super) output: PathBuf,
    pub(super) started: Instant,
    pub(super) requested_first: bool,
    pub(super) ready: bool,
    pub(super) first_paint_ms: Option<f64>,
    pub(super) exit_after_first: bool,
    pub(super) sequence: u64,
    pub(super) pending: Option<(u64, Instant)>,
    pub(super) pending_frames: HashMap<String, (Value, Instant)>,
    pub(super) screenshots_tx: mpsc::Sender<(String, Arc<ColorImage>, Instant)>,
    pub(super) screenshots_rx: mpsc::Receiver<(String, Arc<ColorImage>, Instant)>,
    pub(super) phases: Arc<Mutex<Value>>,
    pub(super) clipboard_requested: bool,
    pub(super) menu_key_handled: Option<bool>,
    #[cfg(target_os = "macos")]
    pub(super) clipboard_backup: Option<super::clipboard::ClipboardBackup>,
}

impl Probe {
    pub(super) fn focus_fixture(&self, context: &Context) {
        context.send_viewport_cmd(ViewportCommand::Visible(true));
        context.send_viewport_cmd(ViewportCommand::Focus);
    }

    pub(super) fn request_screenshot(&self, context: &Context, label: String) {
        let sender = self.screenshots_tx.clone();
        context.request_screenshot(move |image| {
            let _ = sender.send((label, image, Instant::now()));
        });
    }

    pub(super) fn write_report(&self, file: &str, report: Value) -> Result<()> {
        app::atomic_write(
            &self.output.join(file),
            &serde_json::to_vec_pretty(&report)?,
        )
    }

    pub(super) fn report(&self, context: &Context, sequence: u64, elapsed: f64) -> Value {
        let tabs: Vec<Value> = self
            .app
            .tabs
            .iter()
            .map(|tab| json!({"id":tab.id,"title":quick_note::ui::note_title(tab,self.app.locale.text())}))
            .collect();
        let content = context.content_rect();
        let popup = context
            .memory(|memory| memory.area_rect(eframe::egui::Id::unique("note-search-popup")));
        #[cfg(target_os = "macos")]
        let system_menu = super::menu::snapshot();
        #[cfg(not(target_os = "macos"))]
        let system_menu = Value::Null;
        json!({
            "sequence": sequence,
            "system_menu": system_menu,
            "menu_key_handled": self.menu_key_handled,
            "language": self.app.locale.language.code(),
            "saved_label": self.app.locale.text().saved,
            "query": self.app.header.query,
            "content_bounds": [content.left(), content.top(), content.right(), content.bottom()],
            "search_popup_bounds": popup.map(|rect| [rect.left(), rect.top(), rect.right(), rect.bottom()]),
            "theme": self.app.theme.appearance,
            "editor_font": self.app.theme.typography.font_family,
            "markdown_enabled": self.app.markdown_enabled(),
            "settings_open": self.app.settings_open(),
            "local_font": context.fonts(|fonts| fonts.definitions().font_data.get("local-editor").map(|font| json!({"bytes":font.bytes().len(),"face_index":font.index}))),
            "platform_font": context.fonts(|fonts| fonts.definitions().font_data.get("platform-monospace").map(|font| json!({"bytes":font.bytes().len(),"face_index":font.index,"cell_em":font.tweak.monospace_cell_width}))),
            "ui_font": context.fonts(|fonts| fonts.definitions().families.get(&eframe::egui::FontFamily::Proportional).and_then(|family| family.first()).cloned()),
            "editor_focused": context.memory(|memory| memory.focused()) == Some(eframe::egui::Id::unique(("note-editor", &self.app.note.id))),
            "window_focused": context.input(|input| input.focused),
            "elapsed_ms": elapsed,
            "note_id": self.app.note.id,
            "body": self.app.note.body,
            "selection": self.app.note.selection,
            "tabs": tabs,
            "tab_scroll": self.app.header.scroll,
            "dragging_tab": DragAndDrop::has_any_payload(context),
            "pointer_down": context.input(|input| input.pointer.primary_down()),
            "pending_delete": self.app.pending_delete,
            "dirty": self.app.dirty,
            "mode": if self.app.mode == Mode::Edit { "edit" } else { "preview" },
            "error": self.app.error.as_ref().map(|error| self.app.locale.error(error.as_ref())),
            "input": "scripted egui events inside a real native eframe window"
        })
    }

    pub(super) fn fail(&self, error: impl ToString) {
        let _ = self.write_report("failure.json", json!({"error":error.to_string()}));
    }
}

impl eframe::App for Probe {
    fn logic(&mut self, context: &Context, _frame: &mut eframe::Frame) {
        if !self.ready || self.pending.is_some() || !self.pending_frames.is_empty() {
            // A fully covered window gets no UI passes or framebuffer captures.
            // Restore the fixture's visibility before acknowledging its input.
            self.focus_fixture(context);
        }
    }

    fn raw_input_hook(&mut self, context: &Context, input: &mut RawInput) {
        self.feed_input(context, input);
    }
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        self.draw_frame(ui, frame);
    }
    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        eframe::App::clear_color(&self.app, visuals)
    }
}
