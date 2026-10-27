use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Rect;
use eframe::egui::Shape;
use egui_kittest::kittest::Queryable;
use quick_note::i18n::Language;
use quick_note::model::Selection;
use quick_note::storage::Result;

#[test]
fn wide_find_uses_one_row_and_replace_uses_two_without_hiding_controls() -> Result<()> {
    for language in [Language::English, Language::Korean] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        app.note.body = "alpha alpha".into();
        app.note.selection = Selection::default();
        app.dirty = true;
        assert!(app.persist());
        let text = app.locale.text().clone();
        let mut harness = fixtures::harness(app, [920.0, 600.0]);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
        harness.run();
        harness.event(Event::Text("alpha".into()));
        harness.run();
        fixtures::capture(
            &mut harness,
            &format!("find-panel-wide-{}", language.code()),
        );
        let query = harness.get_by_label(&text.find_in_note).rect();
        for label in [
            &text.find_match_case,
            &text.find_toggle_replace,
            &text.find_previous,
            &text.find_next,
            &text.find_close,
        ] {
            let rect = harness.get_by_label(label).rect();
            assert!(
                (rect.center().y - query.center().y).abs() <= 1.0,
                "{label}: {rect:?}"
            );
        }
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
        harness.run();
        let replacement = harness.get_by_label(&text.replace_with).rect();
        let query = harness.get_by_label(&text.find_in_note).rect();
        assert!((replacement.top() - query.bottom() - 4.0).abs() <= 1.0);
        assert!(harness.get_by_label(&text.find_next).rect().bottom() <= query.bottom());
        fixtures::capture(
            &mut harness,
            &format!("replace-panel-wide-{}", language.code()),
        );

        harness.ctx.set_zoom_factor(2.0);
        harness.run();
        harness.set_size(eframe::egui::vec2(320.0, 240.0));
        harness.run();
        assert_eq!(
            harness.ctx.content_rect().size(),
            eframe::egui::vec2(320.0, 240.0)
        );
        assert!(harness.get_by_label(&text.find_in_note).is_focused());
        for label in [
            &text.find_in_note,
            &text.replace_with,
            &text.find_match_case,
            &text.find_toggle_replace,
            &text.find_previous,
            &text.find_next,
            &text.find_close,
            &text.replace_one,
            &text.replace_all,
        ] {
            assert!(
                harness
                    .ctx
                    .content_rect()
                    .contains_rect(harness.get_by_label(label).rect()),
                "{label}"
            );
        }
        assert!(
            harness.get_by_label(&text.find_next).rect().top()
                >= harness.get_by_label(&text.replace_with).rect().bottom()
        );
        fixtures::capture(
            &mut harness,
            &format!("replace-panel-minimum-{}", language.code()),
        );
    }
    Ok(())
}

#[test]
fn filled_find_and_replacement_fields_keep_their_visible_labels() -> Result<()> {
    for language in [Language::English, Language::Korean] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        let text = app.locale.text().clone();
        let mut harness = fixtures::harness(app, [320.0, 240.0]);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
        harness.run();
        harness.event(Event::Text("needle".into()));
        harness.run();
        harness.get_by_label(&text.replace_with).focus();
        harness.run();
        harness.event(Event::Text("replacement".into()));
        harness.run();
        for (name, label) in [
            (&text.find_in_note, &text.find_text),
            (&text.replace_with, &text.replace_with),
        ] {
            let field = harness.get_by_label(name).rect();
            let label_rect = harness
                .output()
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    Shape::Text(painted) if painted.galley.job.text == *label => {
                        let rect = Rect::from_min_size(painted.pos, painted.galley.size());
                        Some((rect, shape.clip_rect))
                    }
                    _ => None,
                })
                .expect("a filled field still paints its purpose label");
            assert!(field.contains_rect(label_rect.0), "{label}");
            assert!(label_rect.1.contains_rect(label_rect.0), "{label}");
        }
        fixtures::capture(
            &mut harness,
            &format!("find-labels-filled-{}", language.code()),
        );
    }
    Ok(())
}

#[test]
fn find_status_distinguishes_empty_queries_no_matches_and_replacement_counts() -> Result<()> {
    for (language, no_matches) in [
        (Language::English, "No matches"),
        (Language::Korean, "결과 없음"),
    ] {
        for count in [1, 2] {
            let (_directory, mut app) = fixtures::fixture()?;
            app.set_language(language, &Context::default())?;
            app.note.body = vec!["alpha"; count].join(" ");
            app.note.selection = Selection::default();
            app.dirty = true;
            assert!(app.persist());
            let text = app.locale.text().clone();
            let mut harness = fixtures::harness(app, [920.0, 600.0]);
            harness.run();
            harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::ALT, Key::F);
            harness.run();
            assert!(harness.query_by_label("0 of 0").is_none());
            assert!(harness.query_by_label("0개 중 0번째").is_none());
            assert!(harness.query_by_label(no_matches).is_none());
            harness.event(Event::Text("absent".into()));
            harness.run();
            harness.get_by_label(no_matches);
            harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
            harness.event(Event::Text("alpha".into()));
            harness.run();
            harness.get_by_label(&text.replace_with).focus();
            harness.run();
            harness.event(Event::Text("beta".into()));
            harness.run();
            harness.get_by_label(&text.replace_all).click();
            harness.run();
            let expected = if language == Language::English {
                format!("Replaced {count}")
            } else {
                format!("{count}개를 바꿨어요")
            };
            harness.get_by_label(&expected);
            assert_eq!(harness.state().note.body, vec!["beta"; count].join(" "));
        }
    }
    Ok(())
}
