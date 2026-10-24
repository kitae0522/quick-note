use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fontique::Collection;
use fontique::CollectionOptions;
use quick_note::app::QuickNoteApp;
use quick_note::i18n::Language;
use quick_note::storage::Result;
use std::thread;
use std::time::Duration;
use std::time::Instant;

fn local_font() -> String {
    let mut fonts = Collection::new(CollectionOptions::default());
    ["Arial", "Liberation Sans", "DejaVu Sans", "Segoe UI"]
        .into_iter()
        .find(|name| fonts.family_id(name).is_some())
        .expect("the desktop must have a proportional font installed")
        .into()
}

fn wait_for(harness: &mut Harness<'_, QuickNoteApp>, label: &str) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while harness.query_by_label(label).is_none() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
        harness.step();
    }
    harness.get_by_label(label);
}

#[test]
fn keyboard_font_search_applies_real_font_and_restores_it_after_restart() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let name = local_font();
    let body = app.note.body.clone();
    let selection = app.note.selection.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose font…").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    wait_for(&mut harness, &name);
    assert!(harness.get_by_label("Search fonts").is_focused());
    harness.event(Event::Text(name.clone()));
    harness.run();
    harness.key_press(Key::ArrowDown);
    harness.run();
    harness.key_press(Key::ArrowUp);
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().theme.typography.font_family, name);
    assert_eq!(
        harness.state().store.setting("editorFont")?.as_deref(),
        Some(name.as_str())
    );
    assert!(
        harness
            .ctx
            .fonts(|fonts| fonts.definitions().font_data.contains_key("local-editor"))
    );
    fixtures::capture(&mut harness, "local-font-picker");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("Note content").is_focused());
    assert_eq!(harness.state().note.body, body);
    assert_eq!(harness.state().note.selection, selection);
    fixtures::capture(&mut harness, "local-font-editor");
    drop(harness);

    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(app.theme.typography.font_family, name);
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !harness
        .ctx
        .fonts(|fonts| fonts.definitions().font_data.contains_key("local-editor"))
        && Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(5));
        harness.step();
    }
    assert!(
        harness
            .ctx
            .fonts(|fonts| fonts.definitions().font_data.contains_key("local-editor"))
    );
    assert_eq!(harness.state().note.body, body);
    Ok(())
}

#[test]
fn missing_saved_font_keeps_writing_available_and_can_be_reset_in_korean() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    let missing = "Missing Local Font a972fe";
    app.store.set_setting("editorFont", missing)?;
    app.theme.typography.font_family = missing.into();
    app.set_language(Language::Korean, &Context::default())?;
    let body = app.note.body.clone();
    let error = format!(
        "이 컴퓨터에서 {missing} 글꼴을 사용할 수 없어요. 설정에서 설치된 글꼴을 선택해 주세요."
    );
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    wait_for(&mut harness, &error);
    assert!(harness.get_by_label("노트 내용").is_focused());
    assert_eq!(harness.state().note.body, body);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("글꼴 선택…").click();
    harness.run();
    harness.get_by_label("테마 기본 글꼴 사용").click();
    harness.run();
    assert_eq!(harness.state().theme.typography.font_family, "monospace");
    assert!(harness.query_by_label(&error).is_none());
    fixtures::capture(&mut harness, "local-font-picker-ko");
    Ok(())
}

#[test]
fn korean_composition_in_font_search_does_not_select_or_edit_a_note() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let body = app.note.body.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose font…").click();
    harness.run();
    wait_for(&mut harness, &local_font());
    harness.event(Event::Ime(ImeEvent::Preedit {
        text: "글".into(),
        active_range_chars: None,
    }));
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().theme.typography.font_family, "monospace");
    assert!(harness.state().store.setting("editorFont")?.is_none());
    assert_eq!(harness.state().note.body, body);
    Ok(())
}

#[test]
fn font_picker_remains_keyboard_operable_in_the_minimum_korean_window() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_language(Language::Korean, &Context::default())?;
    let name = local_font();
    let mut harness = fixtures::harness(app, [320.0, 240.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("글꼴 선택…").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    wait_for(&mut harness, &name);
    assert!(harness.get_by_label("글꼴 검색").is_focused());
    harness.event(Event::Text(name.clone()));
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().theme.typography.font_family, name);
    fixtures::capture(&mut harness, "local-font-picker-narrow-ko");
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.get_by_label("노트 내용").is_focused());
    Ok(())
}
