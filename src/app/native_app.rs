//! Native event-loop work continues when the resident application's window is hidden.
use super::QuickNoteApp;
use eframe::egui::Context;
use eframe::egui::Rangef;
use eframe::egui::Ui;
use eframe::egui::ViewportCommand;
use eframe::egui::Visuals;

impl QuickNoteApp {
    pub(super) fn show_window(context: &Context) {
        #[cfg(target_os = "macos")]
        if let Some(main) = objc2::MainThreadMarker::new() {
            let app = objc2_app_kit::NSApplication::sharedApplication(main);
            app.unhide(None);
            app.activate();
        }
        context.send_viewport_cmd(ViewportCommand::Visible(true));
        context.send_viewport_cmd(ViewportCommand::Minimized(false));
        context.send_viewport_cmd(ViewportCommand::Focus);
    }

    /// Drains native events even when eframe skips rendering an occluded window.
    pub fn background(&mut self, context: &Context) {
        self.receive_capture(context);
        self.receive_menu(context, true);
        self.update_resident(context);
    }
}

impl eframe::App for QuickNoteApp {
    fn logic(&mut self, context: &Context, _frame: &mut eframe::Frame) {
        self.background(context);
    }

    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        frame.set_traffic_lights_position(
            ui.ctx(),
            Rangef::new(0.0, self.theme.layout.toolbar_height),
            12.0,
        );
        self.show(ui);
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        self.theme.palette().background.to_normalized_gamma_f32()
    }
}
