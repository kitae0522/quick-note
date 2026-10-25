//! Geometry and accessible candidate state of the responsive font workspace.
use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Role;
use egui_kittest::kittest::NodeT as _;
use egui_kittest::kittest::Queryable;
use quick_note::i18n::Language;
use quick_note::storage::Result;
use std::thread;
use std::time::Duration;
use std::time::Instant;

#[test]
fn font_candidates_follow_their_heading_without_expanding_the_first_row() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 720.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose fonts…").click();
    harness.run();
    let heading = harness.get_by_label("Fonts").rect();
    let first = harness
        .get_by_role_and_label(Role::CheckBox, "SF Pro")
        .rect();
    assert!(
        first.top() >= heading.bottom() && first.top() - heading.bottom() <= 16.0,
        "heading {heading:?}, first row {first:?}"
    );
    let next = harness
        .get_by_role_and_label(Role::CheckBox, "Pretendard")
        .rect();
    assert!(
        (next.top() - first.top() - 40.0).abs() <= 1.0,
        "{first:?}, {next:?}"
    );
    Ok(())
}

#[test]
fn preview_and_selection_limit_stay_visible_when_browsing_a_full_stack() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    app.set_editor_fonts(
        Some(&[
            "monospace".into(),
            "Pretendard".into(),
            "Noto Sans CJK".into(),
            "Geist".into(),
            "Geist Mono".into(),
            "Geist Medium".into(),
            "Geist Bold".into(),
            "Noto Emoji".into(),
        ]),
        &Context::default(),
    )?;
    let sample = app.locale.text().font_preview_sample.clone();
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose fonts…").click();
    harness.run();
    let preview = harness.get_by_label(&sample).rect();
    assert!(
        harness.ctx.content_rect().contains_rect(preview),
        "{preview:?}"
    );
    let limit = harness
        .get_by_label("8 fonts selected. Remove one to add another.")
        .rect();
    harness.event(Event::Text("Noto".into()));
    harness.run();
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(harness.get_by_label(&sample).rect(), preview);
    assert_eq!(
        harness
            .get_by_label("8 fonts selected. Remove one to add another.")
            .rect(),
        limit
    );
    assert!(harness.ctx.content_rect().contains_rect(limit));
    fixtures::capture(&mut harness, "font-workspace-full-stack");
    Ok(())
}

#[test]
fn arrow_navigation_exposes_the_active_candidate_without_checking_it() -> Result<()> {
    let (_directory, app) = fixtures::fixture()?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
    harness.run();
    harness.get_by_label("Choose fonts…").click();
    harness.run();
    let first = harness
        .get_by_label("Search fonts")
        .accesskit_node()
        .active_descendant()
        .expect("search must expose its current candidate")
        .id();
    harness.key_press(Key::ArrowDown);
    harness.run();
    let search = harness.get_by_label("Search fonts").accesskit_node();
    let candidate = search
        .active_descendant()
        .expect("arrow navigation keeps a candidate");
    assert_ne!(candidate.id(), first);
    assert_eq!(candidate.label().as_deref(), Some("Pretendard"));
    assert_eq!(candidate.role(), Role::CheckBox);
    assert_eq!(
        candidate.toggled(),
        Some(eframe::egui::accesskit::Toggled::False)
    );
    assert!(
        harness
            .ctx
            .memory(|memory| memory.has_focus(Id::unique("font-search")))
    );
    assert_eq!(
        harness.state().theme.typography.font_families(),
        ["monospace"]
    );
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.event(Event::Text("No such font 387f".into()));
    harness.run();
    assert!(
        harness
            .get_by_label("Search fonts")
            .accesskit_node()
            .active_descendant()
            .is_none()
    );
    Ok(())
}

#[test]
fn compact_font_navigation_leaves_space_for_multiple_results_in_both_languages() -> Result<()> {
    for (language, zoom) in [(Language::English, 1.0), (Language::Korean, 2.0)] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_language(language, &Context::default())?;
        let choose = app.locale.text().choose_font.clone();
        let search = app.locale.text().search_fonts.clone();
        let loading = app.locale.text().loading_fonts.clone();
        let mut harness = fixtures::harness(app, [320.0 * zoom, 240.0 * zoom]);
        harness.ctx.set_zoom_factor(zoom);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
        harness.run();
        harness.get_by_label(&choose).focus();
        harness.run();
        harness.key_press(Key::Enter);
        harness.run();
        let deadline = Instant::now() + Duration::from_secs(3);
        while harness.query_all_by_label(&loading).next().is_some() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
            harness.step();
        }
        assert!(harness.query_all_by_label(&loading).next().is_none());
        let rect = harness.get_by_label(&search).rect();
        assert!(rect.top() <= 100.0, "compact search {rect:?}");
        let visible = harness
            .query_all_by_role(Role::CheckBox)
            .filter(|node| harness.ctx.content_rect().contains_rect(node.rect()))
            .count();
        fixtures::capture(
            &mut harness,
            &format!("font-workspace-compact-{}", language.code()),
        );
        let rows: Vec<_> = harness
            .query_all_by_role(Role::CheckBox)
            .take(3)
            .map(|node| (node.accesskit_node().label(), node.rect()))
            .collect();
        assert!(
            visible >= 2,
            "only {visible} complete font rows fit; search {rect:?}, rows {rows:?}"
        );
    }
    Ok(())
}

#[test]
fn intermediate_font_layouts_keep_the_preview_and_a_candidate_inside_the_window() -> Result<()> {
    for size in [[320.0, 600.0], [620.0, 520.0], [920.0, 400.0]] {
        let (_directory, mut app) = fixtures::fixture()?;
        app.set_editor_fonts(
            Some(&[
                "monospace".into(),
                "Pretendard".into(),
                "Noto Sans CJK".into(),
                "Geist".into(),
                "Geist Mono".into(),
                "Geist Medium".into(),
                "Geist Bold".into(),
                "Noto Emoji".into(),
            ]),
            &Context::default(),
        )?;
        let sample = app.locale.text().font_preview_sample.clone();
        let mut harness = fixtures::harness(app, size);
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::Comma);
        harness.run();
        harness.get_by_label("Choose fonts…").focus();
        harness.run();
        harness.key_press(Key::Enter);
        harness.run();
        let candidate = harness
            .get_by_role_and_label(Role::CheckBox, "SF Pro")
            .rect();
        assert!(
            harness.ctx.content_rect().contains_rect(candidate),
            "{size:?}: {candidate:?}"
        );
        if size[1] >= 520.0 {
            let preview = harness.get_by_label(&sample).rect();
            assert!(
                harness.ctx.content_rect().contains_rect(preview),
                "{size:?}: {preview:?}"
            );
        }
        fixtures::capture(
            &mut harness,
            &format!("font-workspace-{}-{}", size[0], size[1]),
        );
    }
    Ok(())
}
