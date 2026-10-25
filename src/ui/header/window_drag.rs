//! Only explicitly empty header space hands a pointer gesture to the window manager.
use eframe::egui::PointerButton;
use eframe::egui::Response;
use eframe::egui::ViewportCommand;

pub(super) fn hand_off(response: Response) {
    if response.drag_started_by(PointerButton::Primary) {
        let context = &response.ctx;
        // Native window tracking can consume mouse-up; egui no longer owns this gesture.
        context.stop_dragging();
        context.input_mut(|input| input.pointer = Default::default());
        context.send_viewport_cmd(ViewportCommand::StartDrag);
    }
}
