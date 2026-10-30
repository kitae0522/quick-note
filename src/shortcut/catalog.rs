//! Action identifiers and localized labels are independent of assigned keys.
use super::Command;
use crate::i18n::Messages;

pub(crate) struct Action {
    pub id: &'static str,
    pub command: Command,
    pub label: fn(&Messages) -> &str,
}

const fn action(id: &'static str, command: Command, label: fn(&Messages) -> &str) -> Action {
    Action { id, command, label }
}

pub(crate) const ACTIONS: &[Action] = &[
    action("new_note", Command::New, |m| &m.new_note),
    action("close_tab", Command::CloseTab, |m| &m.close_note),
    action("recently_deleted", Command::Trash, |m| &m.recently_deleted),
    action("new_capture", Command::NewCapture, |m| &m.capture.new_note),
    action("capture_clipboard", Command::CaptureClipboard, |m| {
        &m.capture.clipboard
    }),
    action("quick_capture", Command::Capture, |m| &m.quick_capture),
    action("focus_mode", Command::FocusMode, |m| &m.focus_mode),
    action("pin_window", Command::PinWindow, |m| &m.pin_window),
    action("delete_note", Command::Delete, |m| &m.shortcut_delete),
    action("quit_app", Command::Quit, |m| &m.shortcut_quit),
    action("close_window", Command::Close, |m| &m.shortcut_close),
    action("search_notes", Command::Search, |m| &m.find_note),
    action("find_in_note", Command::Find, |m| &m.find_in_note),
    action("replace_in_note", Command::Replace, |m| &m.replace_in_note),
    action("find_next", Command::FindNext, |m| &m.find_next),
    action("find_previous", Command::FindPrevious, |m| &m.find_previous),
    action("open_settings", Command::Options, |m| &m.options),
    action("keyboard_shortcuts", Command::Help, |m| {
        &m.keyboard_shortcuts
    }),
    action("toggle_preview", Command::Preview, |m| &m.shortcut_preview),
    action("save_note", Command::Save, |m| &m.retry_save),
    action("export_note", Command::ExportNote, |m| &m.export_note),
    action("reload_theme", Command::ReloadTheme, |m| &m.reload_theme),
    action("import_theme", Command::ImportTheme, |m| &m.import_theme),
    action("export_theme", Command::ExportTheme, |m| &m.export_theme),
    action("open_themes_folder", Command::ThemesFolder, |m| {
        &m.open_themes_folder
    }),
    action("toggle_theme", Command::ToggleTheme, |m| {
        &m.shortcut_toggle_theme
    }),
    action("next_note", Command::NextNote, |m| &m.shortcut_next),
    action("previous_note", Command::PreviousNote, |m| {
        &m.shortcut_previous
    }),
    action("cycle_focus", Command::CycleFocus, |m| &m.shortcut_focus),
    action("move_tab_left", Command::MoveTabLeft, |m| &m.move_tab_left),
    action("move_tab_right", Command::MoveTabRight, |m| {
        &m.move_tab_right
    }),
    action("minimize_window", Command::Minimize, |m| &m.minimize_window),
    action("maximize_window", Command::Maximize, |m| {
        &m.shortcut_maximize
    }),
    action("open_note_1", Command::SelectNote(0), |m| {
        &m.shortcut_open_note
    }),
    action("open_note_2", Command::SelectNote(1), |m| {
        &m.shortcut_open_note
    }),
    action("open_note_3", Command::SelectNote(2), |m| {
        &m.shortcut_open_note
    }),
    action("open_note_4", Command::SelectNote(3), |m| {
        &m.shortcut_open_note
    }),
    action("open_note_5", Command::SelectNote(4), |m| {
        &m.shortcut_open_note
    }),
    action("open_note_6", Command::SelectNote(5), |m| {
        &m.shortcut_open_note
    }),
    action("open_note_7", Command::SelectNote(6), |m| {
        &m.shortcut_open_note
    }),
    action("open_note_8", Command::SelectNote(7), |m| {
        &m.shortcut_open_note
    }),
    action("open_last_note", Command::SelectNote(usize::MAX), |m| {
        &m.shortcut_last_note
    }),
    action("reload_shortcuts", Command::ReloadShortcuts, |m| {
        &m.reload_shortcuts
    }),
    action("open_shortcuts_folder", Command::ShortcutsFolder, |m| {
        &m.open_shortcuts_folder
    }),
];
