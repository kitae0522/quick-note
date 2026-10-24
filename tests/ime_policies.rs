// Compile the exact native-backend policies so callback regressions run on every OS.
#[path = "../vendor/winit/src/platform_impl/macos/ime_commit.rs"]
mod ime_commit;
#[path = "../vendor/winit/src/platform_impl/macos/ime_startup.rs"]
mod ime_startup;
#[path = "ime_policy/line_delete.rs"]
mod line_delete;
