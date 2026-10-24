//! Short, interruptible feedback; initial frames and reduced motion resolve immediately.
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Id;

pub const DURATION: f32 = 0.12;

pub fn configure(context: &Context, reduced: bool) {
    context.all_styles_mut(|style| {
        style.animation_time = if reduced { 0.0 } else { DURATION };
    });
    context.clear_animations();
}

#[must_use]
pub fn transition(context: &Context, id: Id, target: bool) -> f32 {
    let progress = context.animate_bool_with_time(
        id,
        target,
        context.style_of(context.theme()).animation_time,
    );
    // Keep one coordinate space so reversing mid-transition cannot jump visually.
    egui::emath::easing::cubic_out(progress)
}

/// Hidden surfaces reset without keeping the application repainting for an invisible exit.
#[must_use]
pub fn reveal(context: &Context, id: Id, open: bool) -> f32 {
    if open {
        transition(context, id, true)
    } else {
        context.animate_bool_with_time(id, false, 0.0)
    }
}
