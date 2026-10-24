use super::Mode;
use super::QuickNoteApp;
use crate::storage::Result;
use eframe::egui::Context;
use eframe::egui::RawInput;
use rusqlite::Connection;

fn draw(app: &mut QuickNoteApp, context: &Context) {
    let mut output = context.run_ui(RawInput::default(), |ui| app.show(ui));
    output.textures_delta.clear();
}

#[test]
fn cache_is_lazy_and_off_releases_it_and_blocks_programmatic_preview() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let context = Context::default();
    app.configure(&context);
    app.note.body =
        "# An idea\n\n- Keep it small\n- Keep it safe\n\n```rust\nlet a = 1;\n```".into();
    assert!(app.markdown_enabled());
    assert!(app.markdown_cache.is_none());
    draw(&mut app, &context);
    assert!(app.markdown_cache.is_none());
    app.mode = Mode::Preview;
    draw(&mut app, &context);
    assert!(app.markdown_cache.is_some());
    app.mode = Mode::Edit;
    app.set_markdown_enabled(false)?;
    assert!(
        app.markdown_cache.is_none(),
        "OFF must drop caches left by earlier previews"
    );
    app.mode = Mode::Preview;
    draw(&mut app, &context);
    assert_eq!(app.mode, Mode::Edit);
    assert!(app.markdown_cache.is_none());
    app.set_markdown_enabled(true)?;
    draw(&mut app, &context);
    assert!(
        app.markdown_cache.is_none(),
        "ON alone must not initialize the renderer cache"
    );
    app.mode = Mode::Preview;
    draw(&mut app, &context);
    assert!(app.markdown_cache.is_some());
    Ok(())
}

#[test]
fn failed_preference_write_preserves_preview_cache_setting_and_note() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let context = Context::default();
    app.configure(&context);
    app.note.body = "# Saved\n\n한글 아이디어 💡".into();
    app.store.save(&app.note)?;
    app.mode = Mode::Preview;
    draw(&mut app, &context);
    let connection = Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch(
        "CREATE TRIGGER fail_markdown BEFORE INSERT ON app_state BEGIN \
         SELECT RAISE(ABORT, 'test setting failure'); END;",
    )?;
    assert!(app.set_markdown_enabled(false).is_err());
    assert!(app.markdown_enabled());
    assert_eq!(app.mode, Mode::Preview);
    assert!(app.markdown_cache.is_some());
    assert!(app.store.setting("markdownEnabled")?.is_none());
    assert_eq!(
        app.store.note(&app.note.id)?.expect("saved note").body,
        app.note.body
    );
    Ok(())
}
