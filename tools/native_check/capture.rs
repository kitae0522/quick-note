//! Isolated native-window driver. Production never imports this fixture harness.
use eframe::egui::Id;
use eframe::egui::Rangef;
use eframe::egui::Ui;
use eframe::egui::ViewportCommand;
use serde_json::json;
use std::time::Duration;

use super::probe::Probe;

impl Probe {
    pub(super) fn draw_frame(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        frame.set_traffic_lights_position(
            &context,
            Rangef::new(0.0, self.app.theme.layout.toolbar_height),
            12.0,
        );
        self.app.show(ui);
        if context.input(|input| input.focused) {
            self.queued_events.clear();
        } else if !self.native_mode
            && (!self.ready || self.pending.is_some() || !self.pending_frames.is_empty())
        {
            self.focus_fixture(&context);
        }
        if self
            .phases
            .lock()
            .expect("diagnostic phase mutex is not poisoned")
            .get("first_ui_ms")
            .is_none()
        {
            self.phases
                .lock()
                .expect("diagnostic phase mutex is not poisoned")["first_ui_ms"] =
                json!(self.started.elapsed().as_secs_f64() * 1000.0);
        }
        let screenshots: Vec<_> = self.screenshots_rx.try_iter().collect();
        for (label, image, captured) in screenshots {
            let elapsed = captured.duration_since(self.started).as_secs_f64() * 1000.0;
            if let Some((mut report, started)) = self.pending_frames.remove(&label) {
                report["frame_captured_ms"] =
                    json!(captured.duration_since(started).as_secs_f64() * 1000.0);
                let _ = self.write_report("reply.json", report);
            }
            self.first_paint_ms.get_or_insert(elapsed);
            if !self.ready && context.input(|input| input.focused) {
                self.ready = true;
                let editor_focused = context.memory(|memory| memory.focused())
                    == Some(Id::unique(("note-editor", &self.app.note.id)));
                let phases = self
                    .phases
                    .lock()
                    .expect("diagnostic phase mutex is not poisoned")
                    .clone();
                let _ = self.write_report(
                    "startup.json",
                    json!({
                        "first_frame_captured_ms":elapsed,
                        "first_paint_captured_ms":self.first_paint_ms,
                        "editor_focused":editor_focused,
                        "window_focused":context.input(|input|input.focused),
                        "utf8":true,
                        "language":self.app.locale.language.code(),
                        "tabs":self.app.tabs.len(),
                        "measurement":"main entry to a captured native framebuffer with \
                                window and editor focused; includes GPU readback"
                        ,"phases":phases
                    }),
                );
            }
            let bytes: Vec<_> = image
                .pixels
                .iter()
                .flat_map(|pixel| pixel.to_array())
                .collect();
            if let Err(error) = image::save_buffer(
                self.output.join(format!("{label}.png")),
                &bytes,
                image.width() as u32,
                image.height() as u32,
                image::ColorType::Rgba8,
            ) {
                self.fail(error);
            }
            if self.exit_after_first && self.ready {
                context.send_viewport_cmd(ViewportCommand::Close);
            }
        }
        if !self.requested_first {
            self.requested_first = true;
            self.request_screenshot(&context, "startup".to_owned());
        }
        if !self.ready && self.first_paint_ms.is_some() && context.input(|input| input.focused) {
            self.request_screenshot(&context, "input-ready".to_owned());
        }
        if !self.ready && self.started.elapsed() > Duration::from_secs(10) {
            self.fail("The native window did not receive keyboard focus within 10 seconds.");
            context.send_viewport_cmd(ViewportCommand::Close);
        }
        if context.input(|input| input.focused)
            && let Some((sequence, start)) = self.pending.take()
        {
            let label = format!("action-{sequence:03}");
            let report = self.report(&context, sequence, start.elapsed().as_secs_f64() * 1000.0);
            let _ = self.write_report("pending-frame.json", report.clone());
            self.pending_frames.insert(label.clone(), (report, start));
            self.request_screenshot(&context, label);
        }
        if !self.ready || !self.pending_frames.is_empty() {
            // Drive GPU readback without waiting for the production editor's caret blink.
            context.request_repaint_after(Duration::from_millis(1));
        }
    }
}
