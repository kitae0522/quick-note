//! Menu commands share application actions and the focused text widget's editing path.
use crate::shortcut::Command;
use eframe::egui::Key;
use eframe::egui::KeyboardShortcut;
use eframe::egui::Modifiers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditCommand {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}

impl EditCommand {
    #[must_use]
    pub fn keys(self) -> KeyboardShortcut {
        let key = match self {
            Self::Undo | Self::Redo => Key::Z,
            Self::Cut => Key::X,
            Self::Copy => Key::C,
            Self::Paste => Key::V,
            Self::SelectAll => Key::A,
        };
        KeyboardShortcut::new(
            if self == Self::Redo {
                Modifiers::COMMAND | Modifiers::SHIFT
            } else {
                Modifiers::COMMAND
            },
            key,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    App(Command),
    Edit(EditCommand),
    Fonts,
    #[cfg(target_os = "macos")]
    Hide,
    #[cfg(target_os = "macos")]
    HideOthers,
    #[cfg(target_os = "macos")]
    ShowAll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MenuState {
    pub blocked: bool,
    pub composing: bool,
    pub editing: bool,
    pub markdown: bool,
    #[cfg(target_os = "macos")]
    pub preview: bool,
    #[cfg(target_os = "macos")]
    pub settings: bool,
    pub multiple_notes: bool,
}

impl MenuAction {
    pub(crate) fn enabled(self, state: MenuState) -> bool {
        #[cfg(target_os = "macos")]
        if matches!(self, Self::Hide | Self::HideOthers | Self::ShowAll) {
            return true;
        }
        if state.blocked || state.composing {
            return false;
        }
        match self {
            Self::Edit(_) => state.editing,
            Self::App(Command::Preview) => state.markdown,
            Self::App(
                Command::NextNote
                | Command::PreviousNote
                | Command::MoveTabLeft
                | Command::MoveTabRight,
            ) => state.multiple_notes,
            _ => true,
        }
    }
}
