use crate::fixtures;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::kittest::Queryable;
use quick_note::model::Selection;
use quick_note::storage::Result;

#[test]
fn finding_text_navigates_unicode_matches_without_changing_the_note() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body = "💡 alpha 한글 alpha ALPHA alpha".into();
    app.note.selection = Selection::default();
    app.dirty = true;
    assert!(app.persist());
    let original = app.note.body.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    assert!(harness.get_by_label("Find in note").is_focused());
    harness.event(Event::Text("alpha".into()));
    harness.run();
    harness.get_by_label("1 of 4");
    assert_eq!(harness.state().note.selection.char_range(&original), (2, 7));
    harness.key_press(Key::Enter);
    harness.run();
    harness.get_by_label("2 of 4");
    assert_eq!(
        harness.state().note.selection.char_range(&original),
        (11, 16)
    );
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Enter);
    harness.run();
    harness.get_by_label("1 of 4");
    assert_eq!(harness.state().note.body, original);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    Ok(())
}

#[test]
fn replacing_all_is_literal_saved_and_one_undo_step() -> Result<()> {
    let (directory, mut app) = fixtures::fixture()?;
    let original = "💡 alpha 한글 ALPHA\nalpha";
    app.note.body = original.into();
    app.note.selection = Selection::default();
    app.dirty = true;
    assert!(app.persist());
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
    harness.run();
    harness.event(Event::Text("alpha".into()));
    harness.run();
    for label in [
        "Match case",
        "Show replace",
        "Next match",
        "Previous match",
        "Close find",
        "Replace with",
    ] {
        harness.key_press(Key::Tab);
        harness.run();
        assert!(harness.get_by_label(label).is_focused(), "{label}");
    }
    harness.event(Event::Text("{$1} 한💡".into()));
    harness.run();
    harness.get_by_label("Replace all").click();
    harness.run();
    let expected = "💡 {$1} 한💡 한글 {$1} 한💡\n{$1} 한💡";
    assert_eq!(harness.state().note.body, expected);
    assert!(!harness.state().dirty);
    harness.get_by_label("Replaced 3");
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, original);
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    harness.run();
    assert_eq!(harness.state().note.body, expected);
    drop(harness);
    let restored = quick_note::app::QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(restored.note.body, expected);
    Ok(())
}

#[test]
fn case_selection_and_previous_navigation_work_in_the_minimum_korean_window() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_language(
        quick_note::i18n::Language::Korean,
        &eframe::egui::Context::default(),
    )?;
    app.note.body = "한글 alpha ALPHA alpha".into();
    app.note.selection = Selection::from_char_range(&app.note.body, 3, 8);
    app.dirty = true;
    assert!(app.persist());
    let mut harness = fixtures::harness(app, [640.0, 480.0]);
    harness.ctx.set_zoom_factor(2.0);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    harness.get_by_label("3개 중 1번째");
    harness.get_by_label("대소문자 구분").click();
    harness.run();
    harness.get_by_label("2개 중 1번째");
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::G);
    harness.run();
    harness.get_by_label("2개 중 2번째");
    for label in [
        "노트 안에서 찾기",
        "대소문자 구분",
        "찾기 닫기",
        "이전 결과",
        "다음 결과",
    ] {
        assert!(
            harness
                .ctx
                .content_rect()
                .contains_rect(harness.get_by_label(label).rect()),
            "{label}"
        );
    }
    harness.get_by_label("바꾸기 표시").click();
    harness.run();
    for label in ["바꿀 내용", "바꾸기", "모두 바꾸기"] {
        assert!(
            harness
                .ctx
                .content_rect()
                .contains_rect(harness.get_by_label(label).rect()),
            "{label}"
        );
    }
    fixtures::capture(&mut harness, "find-replace-minimum-ko");
    assert_eq!(harness.state().note.body, "한글 alpha ALPHA alpha");
    Ok(())
}

#[test]
fn finding_a_distant_match_reveals_it_without_moving_focus_out_of_the_query() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body = "A line of text.\n".repeat(300) + "A distant needle.\n";
    app.note.selection = Selection::default();
    app.dirty = true;
    assert!(app.persist());
    let body = app.note.body.clone();
    let character = body.find("needle").expect("fixture contains the match");
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    harness.event(Event::Text("needle".into()));
    harness.run();
    assert!(harness.get_by_label("Find in note").is_focused());
    let caret = harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            eframe::egui::Shape::Text(text) if text.galley.job.text == body => Some(
                text.galley
                    .pos_from_cursor(eframe::egui::text::CCursor::new(character))
                    .translate(text.pos.to_vec2()),
            ),
            _ => None,
        })
        .expect("source editor is painted");
    assert!(caret.top() >= 44.0, "{caret:?}");
    assert!(
        caret.bottom() < harness.get_by_label("Find in note").rect().top(),
        "{caret:?}"
    );
    fixtures::capture(&mut harness, "find-distant-match");
    Ok(())
}

#[test]
fn find_next_after_closing_the_panel_advances_in_one_press() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body = "one two one two one".into();
    app.note.selection = Selection::default();
    app.dirty = true;
    assert!(app.persist());
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    harness.event(Event::Text("one".into()));
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::G);
    harness.run();
    assert_eq!(
        harness
            .state()
            .note
            .selection
            .char_range(&harness.state().note.body),
        (8, 11)
    );
    Ok(())
}
