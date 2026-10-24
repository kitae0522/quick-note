//! Sample the production UI at 60 Hz; these are GPU frames, not an OS screen recording.
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Theme;
use egui_kittest::Harness;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| ".artifacts/interface-motion".into());
    fs::create_dir_all(&output)?;
    let mut scenes = Vec::new();
    for scene in ["switch", "search", "reduced"] {
        let directory = tempfile::tempdir()?;
        let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
        app.note.body = "# A thought worth keeping\n\nMake a little room for the next idea.".into();
        assert!(app.persist());
        app.set_theme("helium", &Context::default())?;
        let mut harness = Harness::builder()
            .with_size([720.0, 600.0])
            .with_theme(Theme::Light)
            .with_os(egui::os::OperatingSystem::from_target_os())
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(20)
            .with_render_cursor(false)
            .build_eframe(move |creation| {
                app.configure(&creation.egui_ctx);
                app
            });
        harness.run();
        if scene != "reduced" && harness.state().reduced_motion() {
            scenes.push(json!({"scene":scene,"skipped":"OS Reduce Motion is enabled"}));
            continue;
        }
        let context = harness.ctx.clone();
        if scene == "switch" {
            harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
            harness.run();
        }
        if scene == "reduced" {
            harness.state_mut().set_reduced_motion(true, &context)?;
            harness.run();
        }
        let note = harness.state().note.clone();
        let began = context.input(|input| input.time);
        let mut frames = Vec::new();
        for index in 0..=9 {
            if index == 1 {
                if scene == "switch" {
                    harness.state_mut().set_markdown_enabled(false)?;
                } else {
                    harness.state_mut().request_search(&context);
                }
            }
            if index > 0 {
                harness.step();
            }
            let elapsed = (context.input(|input| input.time) - began) * 1000.0;
            let file = format!("{scene}-{index:02}.png");
            harness.render()?.save(output.join(&file))?;
            frames.push(json!({"file":file,"elapsed_ms":elapsed}));
        }
        assert_eq!(harness.state().note.body, note.body);
        assert_eq!(harness.state().note.selection, note.selection);
        harness.step();
        let stable = harness
            .output()
            .viewport_output
            .values()
            .all(|viewport| viewport.repaint_delay != Duration::ZERO);
        assert!(stable, "motion must not keep repainting after settling");
        scenes.push(json!({"scene":scene,"frames":frames,"settled":stable,
            "reduced_motion":harness.state().reduced_motion()}));
    }
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "passed":true,"scenes":scenes,"sample_hz":60,
            "provenance":"Production QuickNoteApp sampled by egui_kittest; fixture notes and scripted actions",
            "caveats":["Simulated UI time; not a physical keyboard session or OS recording"]
        }))?,
    )?;
    println!("{}", output.display());
    Ok(())
}
