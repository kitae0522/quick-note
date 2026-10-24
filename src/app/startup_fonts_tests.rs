//! Native defaults should be prepared while the window starts, before an egui context exists.
use super::QuickNoteApp;
use crate::storage::Result;
use eframe::egui::Context;
use eframe::egui::FontFamily;
use eframe::egui::RawInput;
use std::thread;
use std::time::Duration;
use std::time::Instant;

fn wait_for_fonts(app: &mut QuickNoteApp) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.fonts.names().is_none() {
        app.fonts.poll()?;
        assert!(
            Instant::now() < deadline,
            "font preparation must start before configure"
        );
        thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}

fn first_editor_font(app: &mut QuickNoteApp) -> Option<String> {
    let context = Context::default();
    app.configure(&context);
    let mut output = context.run_ui(RawInput::default(), |ui| app.show(ui));
    output.textures_delta.clear();
    context.fonts(|fonts| {
        fonts
            .definitions()
            .families
            .get(&FontFamily::Name("editor".into()))
            .and_then(|family| family.first())
            .cloned()
    })
}

#[test]
fn defaults_are_ready_before_configure_and_used_in_the_first_ui_pass() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    wait_for_fonts(&mut app)?;
    assert_eq!(
        first_editor_font(&mut app).as_deref(),
        Some("platform-monospace")
    );
    assert!(app.note.body.is_empty());
    assert!(app.error.is_none());
    Ok(())
}

#[test]
fn an_explicit_font_change_wins_over_prepared_startup_defaults() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    wait_for_fonts(&mut app)?;
    app.theme.typography.font_family = "Pretendard".into();
    assert_eq!(first_editor_font(&mut app).as_deref(), Some("Pretendard"));
    assert!(app.note.body.is_empty());
    Ok(())
}
