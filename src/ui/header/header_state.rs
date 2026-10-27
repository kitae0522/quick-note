use crate::ui::FocusTarget;
use eframe::egui::Rect;

#[derive(Clone, Debug, PartialEq)]
pub enum HeaderAction {
    Select { id: String, focus: FocusTarget },
    Reorder { id: String, index: usize },
    FinishTabDrag,
    Delete(String),
    New,
    Search,
    Options,
    Minimize,
    Maximize,
    CloseWindow,
}

#[derive(Default)]
pub struct HeaderState {
    pub dragging: bool,
    pub query: String,
    pub scroll: f32,
    pub reveal_active: bool,
    pub focus_tab: Option<String>,
    pub focus_search: bool,
}

#[must_use]
pub struct HeaderOutput {
    pub action: Option<HeaderAction>,
    pub search_rect: Rect,
    pub search_changed: bool,
    pub search_gained_focus: bool,
}

/// Native chrome determines the header's reserved space and window controls.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowControls {
    Mac,
    Windows,
    Other,
}
impl WindowControls {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Mac
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }
}
