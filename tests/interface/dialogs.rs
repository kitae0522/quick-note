use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use eframe::egui::ViewportEvent;
use eframe::egui::ViewportId;
use egui_kittest::kittest::Queryable;
use quick_note::i18n::Language;
use quick_note::storage::Result;
use rusqlite::Connection;

#[test]
fn confirmation_content_and_actions_have_breathing_room_in_both_languages() -> Result<()> {
    for (language, size, long_title) in [
        (Language::English, [920.0, 600.0], false),
        (Language::Korean, [320.0, 240.0], false),
        (Language::Korean, [320.0, 240.0], true),
    ] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        if long_title {
            app.note.body =
                "긴 제목을 가진 노트의 삭제 확인 창에서 내용이 잘리는지 확인해요 ".repeat(5);
            app.store.save(&app.note)?;
        }
        let cancel_label = app.locale.text().cancel.clone();
        let delete_label = app.locale.text().delete_note.clone();
        let title_label = app.locale.text().delete_title.clone();
        let mut harness = fixtures::harness(app, size);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::W);
        harness.run();
        let dialog = harness
            .ctx
            .memory(|memory| memory.area_rect(Id::unique("delete-note-confirmation")))
            .expect("visible confirmation area");
        let heading = harness.get_by_label(&title_label).rect();
        let cancel = harness.get_by_label(&cancel_label).rect();
        let delete = harness.get_by_label(&delete_label).rect();
        assert!(
            heading.left() - dialog.left() >= 16.0,
            "heading needs a visible inset"
        );
        assert!(
            heading.top() - dialog.top() >= 16.0,
            "title must not touch the top edge"
        );
        assert!(
            dialog.bottom() - delete.bottom() >= 16.0,
            "actions need a bottom inset"
        );
        assert!(
            dialog.right() - delete.right() >= 16.0,
            "actions need a trailing inset"
        );
        assert!(
            delete.left() - cancel.right() >= 10.0,
            "adjacent actions must stay distinct"
        );
        assert!(dialog.left() >= 0.0 && dialog.right() <= size[0]);
        assert!(dialog.top() >= 0.0 && dialog.bottom() <= size[1]);
        assert!(harness.get_by_label(&cancel_label).is_focused());
        fixtures::capture(
            &mut harness,
            if language == Language::English {
                "dialog-delete"
            } else if long_title {
                "dialog-delete-long-title-ko"
            } else {
                "dialog-delete-narrow-ko"
            },
        );
        harness.key_press(Key::Escape);
        harness.run();
        assert_eq!(harness.state().tabs.len(), 3);
    }
    Ok(())
}

#[test]
fn failed_save_dialog_keeps_padded_actions_visible_at_minimum_size() -> Result<()> {
    for language in [Language::English, Language::Korean] {
        let (directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        let keep = app.locale.text().keep_editing.clone();
        let retry = app.locale.text().retry_save.clone();
        let export = app.locale.text().export_text.clone();
        let connection = Connection::open(directory.path().join("notes.sqlite3"))?;
        connection.execute_batch("CREATE TRIGGER fail_save BEFORE UPDATE ON notes BEGIN SELECT RAISE(ABORT, 'test failure'); END;")?;
        app.note.body.push_str("\nUnsaved idea");
        app.dirty = true;
        let body = app.note.body.clone();
        let mut harness = fixtures::harness(app, [320.0, 240.0]);
        harness.run();
        harness
            .input_mut()
            .viewports
            .get_mut(&ViewportId::ROOT)
            .expect("root viewport")
            .events
            .push(ViewportEvent::Close);
        harness.run();
        let bounds = harness
            .ctx
            .memory(|memory| memory.area_rect(Id::unique("unsaved-before-close")))
            .expect("visible failed-save confirmation");
        let modal = harness.get_by_role(Role::Dialog);
        assert!(modal.get_by_label(&keep).is_focused());
        for label in [&keep, &retry, &export] {
            let rect = modal.get_by_label(label).rect();
            assert!(rect.left() - bounds.left() >= 16.0);
            assert!(bounds.right() - rect.right() >= 16.0);
            assert!(bounds.bottom() - rect.bottom() >= 16.0);
            assert!(rect.top() >= 0.0 && rect.bottom() <= 240.0);
        }
        assert!(bounds.top() >= 0.0 && bounds.bottom() <= 240.0);
        fixtures::capture(
            &mut harness,
            if language == Language::English {
                "dialog-save-narrow"
            } else {
                "dialog-save-narrow-ko"
            },
        );
        harness.key_press(Key::Enter);
        harness.run();
        assert_eq!(harness.state().note.body, body);
        assert!(harness.state().dirty);
    }
    Ok(())
}
