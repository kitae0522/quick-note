//! Read the desktop's animation preference only on configuration and window reactivation.
#[must_use]
pub fn reduced_motion() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWorkspace;
        NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion()
    }
    #[cfg(target_os = "windows")]
    {
        use windows::UI::ViewManagement::UISettings;
        // Unknown preferences use instant feedback until the next activation can read them.
        UISettings::new()
            .and_then(|settings| settings.AnimationsEnabled())
            .map_or(true, |enabled| !enabled)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        true
    }
}
