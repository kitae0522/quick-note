use crate::fixtures;
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Rect;
use egui_kittest::kittest::Queryable;
use quick_note::i18n::Language;
use quick_note::storage::Result;

#[test]
fn search_panel_and_its_controls_stay_inside_safe_margins_after_resizing() -> Result<()> {
    for language in [Language::English, Language::Korean] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        let search = app.locale.text().search_notes.clone();
        let help = app.locale.text().search_help.clone();
        let original = app.note.clone();
        let mut harness = fixtures::harness(app, [920.0, 600.0]);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
        harness.run();
        for size in [
            [920.0, 600.0],
            [320.0, 240.0],
            [620.0, 240.0],
            [1440.0, 800.0],
        ] {
            harness.set_size(size.into());
            harness.run();
            let bounds = harness
                .ctx
                .memory(|memory| memory.area_rect(Id::unique("note-search-popup")))
                .expect("visible search panel");
            let safe = Rect::from_min_max(
                egui::pos2(16.0, harness.state().theme.layout.toolbar_height + 8.0),
                egui::pos2(size[0] - 16.0, size[1] - 16.0),
            );
            fixtures::capture(
                &mut harness,
                &format!("search-safe-area-{}-{}", language.code(), size[0]),
            );
            assert!(
                safe.contains_rect(bounds),
                "search panel escaped safe area: {bounds:?} / {safe:?}"
            );
            let footer = harness.get_by_label(&help).rect();
            assert!(footer.left() >= bounds.left() + 15.0);
            assert!(footer.right() <= bounds.right() - 15.0);
            assert!(footer.bottom() <= bounds.bottom() - 15.0);
            assert!(harness.get_by_label(&search).is_focused());
        }
        harness.key_press(Key::Escape);
        harness.run();
        assert!(
            harness
                .get_by_label(if language == Language::English {
                    "Note content"
                } else {
                    "노트 내용"
                })
                .is_focused()
        );
        assert_eq!(harness.state().note.id, original.id);
        assert_eq!(harness.state().note.body, original.body);
        assert_eq!(harness.state().note.selection, original.selection);
    }
    Ok(())
}

#[test]
fn long_search_results_scroll_without_pushing_help_or_selection_outside_the_panel() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    for index in 0..20 {
        app.new_note()?;
        app.note.body = format!("Idea {index:02} {}", "a long note title ".repeat(12));
        assert!(app.persist());
    }
    app.theme.layout.toolbar_height = 56.0;
    let help = app.locale.text().search_help.clone();
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    for _ in 1..harness.state().search_results().len() {
        harness.key_press(Key::ArrowDown);
        harness.run();
    }
    let last = harness
        .state()
        .search_results()
        .last()
        .expect("search fixture");
    let target_id = last.id.clone();
    let label = format!("Open {}", last.title);
    let bounds = harness
        .ctx
        .memory(|memory| memory.area_rect(Id::unique("note-search-popup")))
        .expect("visible search panel");
    let footer = harness.get_by_label(&help).rect();
    let selected = harness.get_by_label(&label).rect();
    fixtures::capture(&mut harness, "search-safe-area-long-results");
    assert!(bounds.bottom() <= 224.0);
    assert!(selected.top() >= bounds.top() + 16.0);
    assert!(selected.bottom() <= footer.top() - 6.0);
    assert!(footer.bottom() <= bounds.bottom() - 15.0);
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().note.id, target_id);
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}
