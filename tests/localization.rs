use eframe::egui::Context;
use quick_note::app::QuickNoteApp;
use quick_note::appearance::Theme;
use quick_note::i18n;
use quick_note::i18n::Language;
use quick_note::i18n::Localizer;
use quick_note::storage::Result;
use serde_json::Value;
use std::fs;

#[test]
fn both_utf8_catalogs_validate_and_format_without_reinterpreting_user_text() -> Result<()> {
    let en = Localizer::bundled(Language::English)?;
    let ko = Localizer::bundled(Language::Korean)?;
    assert_eq!(en.text().cancel, "Cancel");
    assert_eq!(ko.text().cancel, "취소");
    assert_eq!(
        i18n::format(&ko.text().open_note, &[("title", "{error} 💡")]),
        "{error} 💡 열기"
    );
    assert_eq!(i18n::format("{{{title}}}", &[("title", "한글")]), "{한글}");
    Ok(())
}

#[test]
fn missing_unknown_empty_or_mismatched_translations_are_rejected() -> Result<()> {
    let source = include_bytes!("../i18n/ko.json");
    for mutation in [
        "missing",
        "unknown",
        "empty",
        "placeholder",
        "brace",
        "type",
    ] {
        let mut value: Value = serde_json::from_slice(source)?;
        match mutation {
            "missing" => {
                value
                    .as_object_mut()
                    .expect("catalog object")
                    .remove("cancel");
            }
            "unknown" => value["unexpected_key"] = "value".into(),
            "empty" => value["cancel"] = " ".into(),
            "placeholder" => value["delete_body"] = "{wrong}".into(),
            "brace" => value["delete_body"] = "{title".into(),
            "type" => value["cancel"] = 123.into(),
            _ => unreachable!("fixed fixture cases"),
        }
        assert!(
            Localizer::decode(Language::Korean, &serde_json::to_vec(&value)?).is_err(),
            "{mutation}"
        );
    }
    assert!(Localizer::decode(Language::Korean, b"\xff\xfe").is_err());
    Ok(())
}

#[test]
fn editable_json_overrides_load_without_rebuilding_and_reject_invalid_changes() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("i18n");
    fs::create_dir(&path)?;
    let mut catalog: Value = serde_json::from_slice(include_bytes!("../i18n/ko.json"))?;
    catalog["saved"] = "안전하게 보관됨 💡".into();
    fs::write(path.join("ko.json"), serde_json::to_vec(&catalog)?)?;
    assert_eq!(
        Localizer::load(Language::Korean, directory.path())?
            .text()
            .saved,
        "안전하게 보관됨 💡"
    );
    fs::write(path.join("ko.json"), b"not JSON")?;
    assert!(Localizer::load(Language::Korean, directory.path()).is_err());
    Ok(())
}

#[test]
fn installed_overrides_inherit_new_labels_without_losing_validation() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let folder = directory.path().join("i18n");
    fs::create_dir(&folder)?;
    let path = folder.join("ko.json");
    fs::write(
        &path,
        r#"{"saved":"보관됨","switch_mode_hint":"기존 도움말"}"#,
    )?;
    let locale = Localizer::load(Language::Korean, directory.path())?;
    assert_eq!(locale.text().saved, "보관됨");
    assert_eq!(locale.text().reload_shortcuts, "단축키 다시 불러오기");
    fs::write(&path, r#"{"delete_body":"{wrong}"}"#)?;
    assert!(Localizer::load(Language::Korean, directory.path()).is_err());
    fs::write(&path, r#"{"misspelled_label":"value"}"#)?;
    assert!(Localizer::load(Language::Korean, directory.path()).is_err());
    Ok(())
}

#[test]
fn nested_errors_render_in_the_current_language_with_their_causes_intact() -> Result<()> {
    let cause = i18n::message(|m| &m.error_note_missing);
    let error = i18n::message(|m| &m.error_export).cause("error", cause);
    let ko = Localizer::bundled(Language::Korean)?;
    assert!(ko.error(&error).contains("노트를 찾지 못했어요"));
    assert!(!ko.error(&error).contains("Note not found"));
    assert!(error.to_string().contains("Note not found"));
    let mut theme = Theme::builtin("helium");
    theme.colors.background = "bad".into();
    let error = theme.validate().expect_err("invalid theme fixture");
    assert!(ko.error(error.as_ref()).contains("bad 대신"));
    Ok(())
}

#[test]
fn failed_language_save_preserves_the_current_language_and_note() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    let id = app.note.id.clone();
    let connection = rusqlite::Connection::open(directory.path().join("notes.sqlite3"))?;
    connection.execute_batch(
        "CREATE TRIGGER fail_language BEFORE INSERT ON app_state BEGIN \
         SELECT RAISE(ABORT, 'test failure'); END;",
    )?;
    assert!(
        app.set_language(Language::Korean, &Context::default())
            .is_err()
    );
    assert_eq!(app.locale.language, Language::English);
    assert!(app.store.setting("language")?.is_none());
    assert_eq!(app.note.id, id);
    Ok(())
}

#[test]
fn nested_overrides_inherit_defaults_and_validate_each_leaf() -> Result<()> {
    let directory = tempfile::tempdir()?;
    fs::create_dir(directory.path().join("i18n"))?;
    let path = directory.path().join("i18n/ko.json");
    fs::write(&path, r#"{"capture":{"new_note":"빠른 기록"}}"#)?;
    let locale = Localizer::load(Language::Korean, directory.path())?;
    assert_eq!(locale.text().capture.new_note, "빠른 기록");
    assert_eq!(locale.text().capture.clipboard, "클립보드를 메모로 저장");
    for invalid in [
        r#"{"capture":{"new_note":" "}}"#,
        r#"{"capture":{"clipboard_saved":"{wrong}"}}"#,
        r#"{"capture":{"unknown":"value"}}"#,
        r#"{"capture":"value"}"#,
    ] {
        fs::write(&path, invalid)?;
        assert!(Localizer::load(Language::Korean, directory.path()).is_err());
    }
    Ok(())
}
