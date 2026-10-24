use eframe::egui;
use eframe::egui::Theme;
use egui_kittest::Harness;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use quick_note::storage::Store;
use std::path::Path;

const CAPTURE: &str = include_str!("capture.md");

pub(super) fn fixture() -> Result<(tempfile::TempDir, QuickNoteApp)> {
    let directory = tempfile::tempdir()?;
    {
        let mut store = Store::try_new(&directory.path().join("notes.sqlite3"))?;
        let mut first = store.resume()?;
        first.body = CAPTURE.into();
        store.save(&first)?;
        for body in [
            "# Release checklist\n\nShip the small, useful version.\n\n- [x] Save \
                    immediately\n- [ ] Read it once more",
            "# Weekend ideas\n\nCoffee, a walk, and a notebook.\n\nA café with a quiet \
                    corner. 아이디어 💡",
        ] {
            let mut note = store.create_note()?;
            note.body = body.into();
            store.save(&note)?;
        }
        store.activate(&first.id)?;
    }
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    Ok((directory, app))
}

pub(super) fn harness(app: QuickNoteApp, size: [f32; 2]) -> Harness<'static, QuickNoteApp> {
    harness_with_step(app, size, 0.25)
}

pub(super) fn harness_with_step(
    mut app: QuickNoteApp,
    size: [f32; 2],
    step: f32,
) -> Harness<'static, QuickNoteApp> {
    let appearance = if app.theme.appearance == "dark" {
        Theme::Dark
    } else {
        Theme::Light
    };
    Harness::builder()
        .with_size(size)
        .with_theme(appearance)
        .with_os(egui::os::OperatingSystem::from_target_os())
        .with_max_steps(12)
        .with_step_dt(step)
        .with_render_cursor(false)
        .build_eframe(move |context| {
            context.egui_ctx.add_font_provider(std::sync::Arc::new(
                egui_system_font_provider::SystemFontProvider::new(),
            ));
            app.configure(&context.egui_ctx);
            app
        })
}

pub(super) fn capture(harness: &mut Harness<'_, QuickNoteApp>, name: &str) {
    let directory = Path::new(".artifacts/rust-ui");
    std::fs::create_dir_all(directory).expect("isolated verification fixture should be valid");
    harness
        .render()
        .expect("isolated verification fixture should be valid")
        .save(directory.join(format!("{name}.png")))
        .expect("isolated verification fixture should be valid");
}
