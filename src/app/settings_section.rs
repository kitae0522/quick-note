//! Sections of the in-app settings page; none are stored as user notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum SettingsSection {
    General,
    Fonts,
    Shortcuts,
}
