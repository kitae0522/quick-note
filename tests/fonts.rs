use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::RawInput;
use fontique::Collection;
use fontique::CollectionOptions;
use fontique::QueryFamily;
use fontique::QueryStatus;
use fontique::SourceCache;
use quick_note::app::QuickNoteApp;
use quick_note::appearance;
use quick_note::appearance::FontCatalog;
use quick_note::appearance::Theme;
use quick_note::storage::Result;
use std::fs;

#[test]
fn native_font_data_and_collection_face_are_used_by_the_actual_renderer() -> Result<()> {
    let mut system = Collection::new(CollectionOptions::default());
    let names: Vec<String> = system.family_names().map(str::to_owned).collect();
    let name = ["Arial", "Liberation Sans", "DejaVu Sans", "Segoe UI"]
        .into_iter()
        .find(|candidate| names.iter().any(|name| name == candidate))
        .expect("a desktop test system must provide a standard proportional font");
    let mut catalog = FontCatalog::default();
    assert!(
        catalog.names().is_none(),
        "startup must not enumerate fonts eagerly"
    );
    let definitions = catalog.definitions(name)?;
    let native = &definitions.font_data["local-editor"];
    let mut cache = SourceCache::default();
    let mut query = system.query(&mut cache);
    query.set_families([QueryFamily::Named(name)]);
    let mut matched = false;
    query.matches_with(|font| {
        assert_eq!(
            native.index, font.index,
            "TTC face indices must be preserved"
        );
        assert_eq!(native.bytes(), font.blob.as_ref());
        matched = true;
        QueryStatus::Stop
    });
    assert!(matched);
    assert_eq!(catalog.names().expect("loaded catalog").len(), names.len());
    let insensitive = catalog.definitions(&name.to_lowercase())?;
    assert_eq!(insensitive.font_data["local-editor"], *native);
    let context = Context::default();
    context.set_fonts(definitions);
    let mut output = context.run_ui(RawInput::default(), |ui| {
        ui.fonts_mut(|fonts| {
            let local = fonts.layout_no_wrap(
                "iiiiWWWW".into(),
                appearance::editor_font(18.0),
                Color32::BLACK,
            );
            let mono =
                fonts.layout_no_wrap("iiiiWWWW".into(), FontId::monospace(18.0), Color32::BLACK);
            assert!((local.size().x - mono.size().x).abs() > 1.0);
            let mixed = fonts.layout_no_wrap(
                "English 한글 💡".into(),
                appearance::editor_font(18.0),
                Color32::BLACK,
            );
            assert!(mixed.size().x.is_finite() && mixed.size().y > 0.0);
        });
    });
    output.textures_delta.clear();
    let second = catalog.definitions("monospace")?;
    assert!(
        !second.font_data.contains_key("local-editor"),
        "old font bytes must not accumulate"
    );
    assert!(
        second
            .families
            .contains_key(&FontFamily::Name("editor".into()))
    );
    Ok(())
}

#[test]
fn missing_fonts_and_failed_setting_writes_preserve_working_font_and_note() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let context = Context::default();
    app.configure(&context);
    app.note.body = "보관할 아이디어 💡".into();
    app.store.save(&app.note)?;
    app.set_editor_font(Some("Pretendard"), &context)?;
    let theme = app.theme.clone();
    let note = app.note.clone();
    assert!(
        app.set_editor_font(Some("No Such Font 7c7adf24"), &context)
            .is_err()
    );
    assert_eq!(app.theme, theme);
    assert_eq!(
        app.store.setting("editorFont")?.as_deref(),
        Some("Pretendard")
    );
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch(
        "CREATE TRIGGER fail_font BEFORE INSERT ON app_state BEGIN \
         SELECT RAISE(ABORT, 'test font write failure'); END;",
    )?;
    assert!(app.set_editor_font(Some("Geist"), &context).is_err());
    assert_eq!(app.theme, theme);
    assert_eq!(app.note.body, note.body);
    assert_eq!(app.note.selection, note.selection);
    assert_eq!(
        app.store.setting("editorFont")?.as_deref(),
        Some("Pretendard")
    );
    Ok(())
}

#[test]
fn theme_font_override_reset_and_shared_json_keep_the_intended_family() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let context = Context::default();
    app.configure(&context);
    let mut theme = Theme::builtin("helium");
    theme.typography.font_family = "Arial".into();
    fs::write(
        directory.path().join("Themes/local.json"),
        serde_json::to_vec(&theme)?,
    )?;
    // Resolve a native family through a shared theme, independently of the options picker.
    let mut system = Collection::new(CollectionOptions::default());
    if system.family_id("Arial").is_none() {
        theme.typography.font_family = system.family_names().next().expect("desktop fonts").into();
        fs::write(
            directory.path().join("Themes/local.json"),
            serde_json::to_vec(&theme)?,
        )?;
    }
    app.set_theme("local.json", &context)?;
    assert_eq!(
        app.theme.typography.font_family,
        theme.typography.font_family
    );
    app.set_editor_font(Some("Pretendard"), &context)?;
    app.set_theme("graphite", &context)?;
    assert_eq!(app.theme.typography.font_family, "Pretendard");
    let exported = Theme::decode(&serde_json::to_vec(&app.theme)?)?;
    assert_eq!(exported.typography.font_family, "Pretendard");
    app.set_theme("local.json", &context)?;
    app.set_editor_font(None, &context)?;
    assert_eq!(
        app.theme.typography.font_family,
        theme.typography.font_family
    );
    assert_eq!(app.store.setting("editorFont")?.as_deref(), Some(""));
    Ok(())
}
