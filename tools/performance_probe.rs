//! Release CPU timings for production UI and storage paths; fixture directories only.
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::RawInput;
use quick_note::app::Mode;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::hint;
use std::path::PathBuf;
use std::time::Instant;

fn statistics(mut values: Vec<f64>) -> Value {
    values.sort_by(f64::total_cmp);
    json!({"samples": values.len(), "median_ms": values[values.len() / 2],
        "p95_ms": values[((values.len() as f64 * 0.95).ceil() as usize - 1).min(values.len() - 1)],
        "max_ms": values.last(), "mean_ms": values.iter().sum::<f64>() / values.len() as f64})
}

fn keys(key: Key, modifiers: Modifiers) -> Vec<Event> {
    [true, false]
        .map(|pressed| Event::Key {
            key,
            modifiers,
            pressed,
            physical_key: Some(key),
            repeat: false,
        })
        .into()
}

fn frame(app: &mut QuickNoteApp, context: &Context, events: Vec<Event>, time: &mut f64) -> f64 {
    *time += 1.0 / 60.0;
    let started = Instant::now();
    let output = context.run_ui(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(920.0, 600.0),
            )),
            time: Some(*time),
            focused: true,
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    hint::black_box(output);
    started.elapsed().as_secs_f64() * 1000.0
}

fn frames(
    app: &mut QuickNoteApp,
    context: &Context,
    time: &mut f64,
    count: usize,
    event: impl Fn() -> Vec<Event>,
) -> Value {
    statistics(
        (0..count)
            .map(|_| frame(app, context, event(), time))
            .collect(),
    )
}

fn main() -> Result<()> {
    let mut arguments = env::args_os().skip(1);
    let directory = PathBuf::from(arguments.next().expect("isolated fixture directory"));
    let report = PathBuf::from(arguments.next().expect("report path"));
    let started = Instant::now();
    let mut app = QuickNoteApp::try_new(directory)?;
    let startup_ms = started.elapsed().as_secs_f64() * 1000.0;
    let original_body = app.note.body.clone();
    let note_count = app.tabs.len();
    let mut search = Vec::new();
    for index in 0..65 {
        app.header.query = ["idea", "marker", "missing", "small", "café"][index % 5].into();
        let start = Instant::now();
        app.refresh_search()?;
        hint::black_box(app.search_results().len());
        search.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let cold_search_ms = search.remove(0);
    let context = Context::default();
    app.configure(&context);
    let mut time = 0.0;
    for _ in 0..5 {
        frame(&mut app, &context, Vec::new(), &mut time);
    }
    let source_redraw = frames(&mut app, &context, &mut time, 30, Vec::new);
    let cursor = frames(&mut app, &context, &mut time, 20, || {
        keys(Key::ArrowLeft, Modifiers::NONE)
    });
    let typing = frames(&mut app, &context, &mut time, 20, || {
        vec![Event::Text("x".into())]
    });
    assert!(!app.dirty && app.error.is_none());
    app.header.query.clear();
    app.request_search(&context);
    for _ in 0..3 {
        frame(&mut app, &context, Vec::new(), &mut time);
    }
    let search_redraw = frames(&mut app, &context, &mut time, 20, Vec::new);
    frame(
        &mut app,
        &context,
        keys(Key::Escape, Modifiers::NONE),
        &mut time,
    );
    app.mode = Mode::Preview;
    for _ in 0..3 {
        frame(&mut app, &context, Vec::new(), &mut time);
    }
    let markdown = frames(&mut app, &context, &mut time, 12, Vec::new);
    let start = Instant::now();
    app.set_editor_font(Some("Arial"), &context)?;
    let local_font_ms = start.elapsed().as_secs_f64() * 1000.0;
    frame(
        &mut app,
        &context,
        keys(Key::Comma, Modifiers::COMMAND),
        &mut time,
    );
    frame(&mut app, &context, Vec::new(), &mut time);
    for _ in 0..4 {
        frame(
            &mut app,
            &context,
            keys(Key::Tab, Modifiers::NONE),
            &mut time,
        );
    }
    frame(
        &mut app,
        &context,
        keys(Key::Enter, Modifiers::NONE),
        &mut time,
    );
    for _ in 0..3 {
        frame(&mut app, &context, Vec::new(), &mut time);
    }
    assert!(
        context.memory(|memory| memory.has_focus(egui::Id::unique("font-search"))),
        "font-picker benchmark must run on the actual font page"
    );
    let font_picker = frames(&mut app, &context, &mut time, 20, Vec::new);
    let stored = app.store.note(&app.note.id)?.expect("fixture note");
    assert_eq!(stored.body, app.note.body);
    assert_eq!(app.note.body.len(), original_body.len() + 20);
    let result = json!({"notes": note_count, "original_body_bytes": original_body.len(),
        "startup_storage_ms": startup_ms, "cold_search_ms": cold_search_ms,
        "warm_search": statistics(search), "source_redraw": source_redraw,
        "cursor": cursor, "typing_with_durable_save": typing, "search_redraw": search_redraw,
        "markdown_redraw": markdown, "local_font_ms": local_font_ms, "font_picker_redraw": font_picker,
        "measurement": "release CPU wall time in production UI, excluding GPU presentation; isolated SQLite with synchronous FULL and macOS fullfsync"});
    fs::write(report, serde_json::to_vec_pretty(&result)?)?;
    Ok(())
}
