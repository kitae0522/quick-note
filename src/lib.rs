pub mod app;
pub mod appearance;
pub mod i18n;
pub mod menu;
pub mod model;
pub mod native;
pub mod shortcut;
pub mod storage;
pub mod ui;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod capture;
