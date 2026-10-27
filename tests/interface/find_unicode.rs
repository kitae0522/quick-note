use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::model::Selection;
use quick_note::storage::Result;

#[test]
fn unicode_case_replacements_save_original_ranges_and_undo_once() -> Result<()> {
    for (original, query, expected, count) in [
        ("💡 ΟΣ ος οσ 한글", "ΟΣ", "💡 note note note 한글", 3),
        ("💡 Straße STRASSE 한글", "strasse", "💡 note note 한글", 2),
        ("💡 ﬃfif 한글", "fif", "💡 ﬃnote 한글", 1),
    ] {
        let (directory, mut app) = fixtures::fixture()?;
        app.note.body = original.into();
        app.note.selection = Selection::default();
        assert!(app.persist());
        let id = app.note.id.clone();
        let mut harness = fixtures::harness(app, [920.0, 600.0]);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
        harness.run();
        harness.event(Event::Text(query.into()));
        harness.run();
        harness.get_by_label(format!("1 of {count}").as_str());
        harness.get_by_label("Replace with").click();
        harness.event(Event::Text("note".into()));
        harness.run();
        harness.get_by_label("Replace all").click();
        harness.run();
        assert_eq!(harness.state().note.body, expected);
        assert!(!harness.state().dirty);
        harness.key_press(Key::Escape);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
        harness.run();
        assert_eq!(harness.state().note.body, original);
        drop(harness);
        let app = quick_note::app::QuickNoteApp::try_new(directory.path().to_owned())?;
        assert_eq!(app.note.id, id);
        assert_eq!(app.note.body, original);
    }
    Ok(())
}
