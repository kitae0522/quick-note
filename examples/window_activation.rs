//! Native regression check for activation when a GPU renderer reveals a hidden window.
#[cfg(target_os = "macos")]
#[path = "window_activation/mac.rs"]
mod mac;

#[cfg(target_os = "macos")]
fn main() -> quick_note::storage::Result<()> {
    mac::run()
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This activation check requires macOS.");
}
