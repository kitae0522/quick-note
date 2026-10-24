//! Offscreen GPU renders of the production UI with isolated fixture notes; no personal data.
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Theme;
use egui_kittest::Harness;
use quick_note::app::QuickNoteApp;
use quick_note::i18n::Language;
use quick_note::storage::Result;
use quick_note::storage::Store;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn fixture(language: Language, dark: bool) -> Result<(tempfile::TempDir, QuickNoteApp)> {
    let directory = tempfile::tempdir()?;
    {
        let mut store = Store::try_new(&directory.path().join("notes.sqlite3"))?;
        let mut first = store.resume()?;
        first.body = include_str!("../tests/interface/capture.md").into();
        store.save(&first)?;
        for body in [
            "# Release checklist\n\nShip the small, useful version.",
            "# Weekend ideas\n\nCoffee, a walk, and a notebook. 아이디어 💡",
        ] {
            let mut note = store.create_note()?;
            note.body = body.into();
            store.save(&note)?;
        }
        store.activate(&first.id)?;
    }
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    app.set_language(language, &Context::default())?;
    if dark {
        app.set_theme("graphite", &Context::default())?;
    }
    Ok((directory, app))
}

fn main() -> Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| ".artifacts/interface-gallery".into());
    fs::create_dir_all(&output)?;
    let mut reports = Vec::new();
    for (name, size, language, dark, action) in [
        (
            "edit-light",
            [920.0, 600.0],
            Language::English,
            false,
            "edit",
        ),
        ("edit-dark", [920.0, 600.0], Language::English, true, "edit"),
        ("blank", [720.0, 480.0], Language::English, false, "blank"),
        (
            "preview",
            [920.0, 600.0],
            Language::English,
            false,
            "preview",
        ),
        ("search", [920.0, 600.0], Language::English, false, "search"),
        (
            "search-empty",
            [920.0, 600.0],
            Language::English,
            false,
            "empty",
        ),
        (
            "settings",
            [920.0, 720.0],
            Language::English,
            false,
            "settings",
        ),
        (
            "settings-dark",
            [920.0, 720.0],
            Language::English,
            true,
            "settings",
        ),
        ("delete", [920.0, 600.0], Language::English, false, "delete"),
        (
            "narrow-edit",
            [320.0, 240.0],
            Language::Korean,
            false,
            "edit",
        ),
        (
            "narrow-settings",
            [320.0, 360.0],
            Language::Korean,
            false,
            "settings",
        ),
        (
            "narrow-search",
            [320.0, 240.0],
            Language::Korean,
            false,
            "search",
        ),
        (
            "narrow-delete",
            [320.0, 240.0],
            Language::Korean,
            false,
            "delete",
        ),
    ] {
        let (_directory, mut app) = fixture(language, dark)?;
        let theme = if dark { Theme::Dark } else { Theme::Light };
        let mut harness = Harness::builder()
            .with_size(size)
            .with_theme(theme)
            .with_os(egui::os::OperatingSystem::from_target_os())
            .with_max_steps(16)
            .with_render_cursor(false)
            .build_eframe(move |creation| {
                app.configure(&creation.egui_ctx);
                app
            });
        harness.run();
        match action {
            "blank" => harness.key_press_modifiers(Modifiers::COMMAND, Key::N),
            "preview" => harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M),
            "search" | "empty" => harness.key_press_modifiers(Modifiers::COMMAND, Key::P),
            "settings" => harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma),
            "delete" => harness.key_press_modifiers(Modifiers::COMMAND, Key::W),
            _ => {}
        }
        harness.run();
        if action == "empty" {
            harness.event(Event::Text("a missing thought".into()));
            harness.run();
        }
        harness.render()?.save(output.join(format!("{name}.png")))?;
        reports.push(json!({"name":name,"size":size,"language":language.code(),"dark":dark,
            "tabs":harness.state().tabs.len(),"body":harness.state().note.body,
            "settings":harness.state().settings_open(),"pending_delete":harness.state().pending_delete}));
    }
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "passed":true,"cases":reports,
            "provenance":"Production egui widgets rendered to GPU textures by egui_kittest; fixture notes only",
            "caveats":["Offscreen renders exclude AppKit traffic lights, physical keyboard/IME and screen-reader speech"]
        }))?,
    )?;
    println!("{}", output.display());
    Ok(())
}
