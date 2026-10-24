use eframe::egui::ViewportBuilder;

#[must_use]
pub fn options(title: &str) -> eframe::NativeOptions {
    let mut viewport = ViewportBuilder::default()
        .with_title(title)
        .with_app_id("local.quicknote.mac")
        .with_inner_size([920.0, 600.0])
        .with_min_inner_size([320.0, 240.0])
        .with_active(true)
        .with_icon(
            eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png"))
                .expect("bundled app icon is valid PNG"),
        );
    if cfg!(target_os = "macos") {
        viewport = viewport
            .with_fullsize_content_view(true)
            .with_title_shown(false)
            .with_titlebar_shown(false);
    }
    if cfg!(target_os = "windows") {
        viewport = viewport.with_decorations(false);
    }
    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    // Notes have little GPU work: queue one frame to prioritize input latency.
    options.wgpu_options.surface = eframe::SurfaceConfig::LOW_LATENCY;
    options
}
