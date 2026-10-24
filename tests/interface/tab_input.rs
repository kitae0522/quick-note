//! Pointer gestures use separate press, move and release frames like a real tab drag.
use eframe::egui::Event;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use eframe::egui::Pos2;
use egui_kittest::Harness;
use quick_note::app::QuickNoteApp;

pub(super) fn start(harness: &mut Harness<'_, QuickNoteApp>, from: Pos2, to: Pos2) {
    harness.hover_at(from);
    harness.step();
    harness.drag_at(from);
    harness.step();
    harness.hover_at(to);
    harness.step();
}

pub(super) fn finish(harness: &mut Harness<'_, QuickNoteApp>, position: Pos2) {
    // Keep the position available on release; drop_at also emits PointerGone.
    harness.event(Event::PointerButton {
        pos: position,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.run();
}

pub(super) fn ids(harness: &Harness<'_, QuickNoteApp>) -> Vec<String> {
    harness
        .state()
        .tabs
        .iter()
        .map(|note| note.id.clone())
        .collect()
}
