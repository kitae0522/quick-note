use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::MouseWheelSource;
use eframe::egui::MouseWheelUnit;
use eframe::egui::TouchPhase;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::i18n::Language;
use quick_note::storage::Result;

fn reveal_preview(harness: &mut Harness<'_, QuickNoteApp>, label: &str) {
    // Offscreen Markdown is intentionally absent from the accessibility tree until scrolled in.
    for down in [false, true] {
        for _ in 0..20 {
            if harness.query_by_label(label).is_some() {
                return;
            }
            harness.hover_at(harness.ctx.content_rect().center());
            harness.event(Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                source: MouseWheelSource::Wheel,
                phase: TouchPhase::Move,
                delta: eframe::egui::vec2(0.0, if down { -120.0 } else { 120.0 }),
                modifiers: Modifiers::NONE,
            });
            harness.run();
        }
    }
    panic!("Markdown label never became reachable by scrolling: {label}");
}

#[test]
fn selecting_korean_updates_the_ui_and_persists_without_changing_notes() -> Result<()> {
    let (directory, app) = fixtures::fixture()?;
    let before: Vec<_> = app
        .store
        .notes()?
        .into_iter()
        .map(|note| (note.id, note.body))
        .collect();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    fixtures::capture(&mut harness, "i18n-english");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("한국어").click();
    harness.run();
    assert_eq!(harness.state().locale.language, Language::Korean);
    harness.get_by_label("언어");
    harness.get_by_label("노트 검색");
    fixtures::capture(&mut harness, "i18n-options-ko");
    harness.get_by_label("노트로 돌아가기").click();
    harness.run();
    harness.get_by_label("노트 내용");
    harness.get_by_label("저장됨");
    fixtures::capture(&mut harness, "i18n-korean");
    assert_eq!(
        harness.state().store.setting("language")?.as_deref(),
        Some("ko")
    );
    for (id, body) in &before {
        assert_eq!(
            harness.state().store.note(id)?.expect("original note").body,
            *body
        );
    }
    drop(harness);
    let app = QuickNoteApp::try_new(directory.path().to_owned())?;
    assert_eq!(app.locale.language, Language::Korean);
    Ok(())
}

#[test]
fn korean_search_confirmation_markdown_and_minimum_layout_use_translated_labels() -> Result<()> {
    for size in [[920.0, 600.0], [320.0, 240.0]] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.note
            .body
            .push_str("\n\n> [!WARNING]\n> Read this once.");
        assert!(app.persist());
        app.set_language(Language::Korean, &Context::default())?;
        let mut harness = fixtures::harness(app, size);
        harness.run();
        for label in ["새 노트", "설정 열기"] {
            let rect = harness.get_by_label(label).rect();
            assert!(rect.left() >= 0.0 && rect.right() <= size[0]);
        }
        fixtures::capture(
            &mut harness,
            if size[0] < 400.0 {
                "i18n-narrow-ko"
            } else {
                "i18n-edit-ko"
            },
        );
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::M);
        harness.run();
        reveal_preview(&mut harness, "경고");
        reveal_preview(&mut harness, "코드 블록 (읽기 전용)");
        harness.get_by_label("코드 블록 (읽기 전용)");
        harness.get_by_label("코드 복사");
        if size[0] > 400.0 {
            let position = harness.get_by_label("코드 복사").rect().center();
            harness.hover_at(position);
            harness.run();
            harness.get_by_label("코드 복사").click();
            harness.run();
            harness.get_by_label("코드를 복사했어요. 다시 복사");
            fixtures::capture(&mut harness, "i18n-preview-ko");
            let context = harness.ctx.clone();
            harness
                .state_mut()
                .set_language(Language::English, &context)?;
            harness.run();
            reveal_preview(&mut harness, "Warning");
            reveal_preview(&mut harness, "Code block (read only)");
            harness.get_by_label("Code block (read only)");
            harness
                .state_mut()
                .set_language(Language::Korean, &context)?;
            harness.run();
            reveal_preview(&mut harness, "경고");
        }
        harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
        harness.run();
        harness.get_by_label("노트 검색").type_text("CAFÉ");
        harness.run();
        assert_eq!(harness.state().search_results().len(), 1);
        harness.key_press(Key::Enter);
        harness.run();
        assert_eq!(harness.state().note.title(), "Weekend ideas");
        harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
        harness.run();
        assert!(harness.get_by_label("취소").is_focused());
        harness.key_press(Key::Enter);
        harness.run();
        assert!(harness.state().pending_delete.is_none());
    }
    Ok(())
}

#[test]
fn only_generated_untitled_names_are_translated_and_searchable() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.new_note()?;
    app.note.body = "Untitled".into();
    assert!(app.persist());
    let literal = app.note.id.clone();
    app.new_note()?;
    let blank = app.note.id.clone();
    app.set_language(Language::Korean, &Context::default())?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.get_by_label("Untitled");
    harness.get_by_label("제목 없음");
    harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
    harness.run();
    harness.get_by_label("노트 검색").type_text("제목 없음");
    harness.run();
    assert_eq!(harness.state().search_results().len(), 1);
    assert_eq!(harness.state().search_results()[0].id, blank);
    harness.key_press(Key::Escape);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
    harness.run();
    assert!(harness.state().pending_delete.is_none());
    assert!(harness.state().store.note(&blank)?.is_none());
    assert_eq!(
        harness
            .state()
            .store
            .note(&literal)?
            .expect("authored title")
            .body,
        "Untitled"
    );
    Ok(())
}
