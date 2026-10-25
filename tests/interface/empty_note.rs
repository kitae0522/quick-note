use crate::fixtures;
use eframe::egui::Shape;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::i18n::Language;
use quick_note::storage::Result;
use quick_note::storage::Store;

fn paints_text(shape: &Shape, text: &str) -> bool {
    match shape {
        Shape::Text(shape) => shape.galley.job.text.contains(text),
        Shape::Vec(shapes) => shapes.iter().any(|shape| paints_text(shape, text)),
        _ => false,
    }
}

#[test]
fn an_empty_note_has_no_placeholder_and_accepts_the_first_character() -> Result<()> {
    for language in [Language::English, Language::Korean] {
        let directory = tempfile::tempdir()?;
        let mut store = Store::try_new(&directory.path().join("notes.sqlite3"))?;
        store.resume()?;
        store.set_setting("language", language.code())?;
        drop(store);
        let app = QuickNoteApp::try_new(directory.path().to_owned())?;
        let hint = app.locale.text().start_writing.clone();
        let label = app.locale.text().note_content.clone();
        let mut harness = fixtures::harness(app, [920.0, 600.0]);
        harness.run();
        assert!(harness.get_by_label(&label).is_focused());
        assert!(
            !harness
                .output()
                .shapes
                .iter()
                .any(|shape| paints_text(&shape.shape, &hint))
        );
        harness.event(eframe::egui::Event::Text("한💡".into()));
        harness.run();
        assert_eq!(harness.state().note.body, "한💡");
        assert!(!harness.state().dirty);
    }
    Ok(())
}
