//! The quick window reuses the current editor and keeps its normal window geometry.
use crate::model::Note;
use crate::shortcut::Command;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Default)]
pub(super) struct CaptureState {
    #[cfg(target_os = "macos")]
    pub login: super::login_item::LoginItem,
    pub compact: bool,
    pub quitting: bool,
    pub hide_pending: bool,
    pub normal_position: Option<Pos2>,
    pub focused: bool,
    pub pinned: bool,
    pub normal_rect: Option<Rect>,
    pub pending: VecDeque<(Command, bool)>,
    pub draft_origin: Option<Note>,
    pub recent_dirty: bool,
    pub recent_due: Option<Instant>,
    pub saved_until: Option<Instant>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub native: Option<crate::capture::GlobalCapture>,
}
