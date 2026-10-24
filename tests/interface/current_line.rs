use crate::fixtures;
use eframe::egui;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use quick_note::app::QuickNoteApp;
use quick_note::appearance::Theme;
use quick_note::model::Selection;
use quick_note::storage::Result;

const INDICATOR: [u8; 4] = [41, 184, 111, 255];

fn fixture(body: &str, caret: usize, theme: &str) -> Result<(tempfile::TempDir, QuickNoteApp)> {
    let (directory, mut app) = fixtures::fixture()?;
    let mut value = serde_json::to_value(Theme::builtin(theme))?;
    value["colors"]["currentLine"] = "#29B86F".into();
    app.theme = Theme::decode(&serde_json::to_vec(&value)?)?;
    app.note.body = body.into();
    app.note.selection = Selection::from_char_range(body, caret, caret);
    app.store.save(&app.note)?;
    Ok((directory, app))
}

fn indicator(harness: &mut Harness<'_, QuickNoteApp>, name: &str) -> Option<Rect> {
    let image = harness.render().expect("render the real editor");
    std::fs::create_dir_all(".artifacts/rust-ui").expect("fixture output directory");
    image
        .save(format!(".artifacts/rust-ui/{name}.png"))
        .expect("fixture screenshot");
    let mut bounds = Rect::NOTHING;
    let scale = harness.ctx.pixels_per_point();
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0 == INDICATOR {
            bounds.extend_with(egui::pos2(x as f32 / scale, y as f32 / scale));
        }
    }
    bounds.is_positive().then_some(bounds)
}

#[test]
fn current_line_follows_keyboard_and_pointer_and_restores_with_the_note() -> Result<()> {
    for theme in ["helium", "graphite"] {
        let body = "First idea\nSecond idea\nThird idea";
        let (_directory, app) = fixture(body, 0, theme)?;
        let original = app.note.id.clone();
        let line_height = app.theme.typography.font_size + app.theme.typography.line_spacing;
        let mut harness = fixtures::harness(app, [640.0, 400.0]);
        harness.run();
        let first = indicator(&mut harness, &format!("current-line-{theme}-first"))
            .expect("the current line must have a gutter background");
        assert!(first.right() < 64.0 && first.width() >= 24.0);
        harness.key_press(Key::ArrowDown);
        harness.key_press(Key::ArrowDown);
        harness.run();
        let third = indicator(&mut harness, &format!("current-line-{theme}-third"))
            .expect("the moved cursor must retain its indicator");
        assert!((third.top() - first.top() - 2.0 * line_height).abs() <= 1.0);
        let position = egui::pos2(100.0, first.top() + 1.5 * line_height);
        harness.event(Event::PointerMoved(position));
        for pressed in [true, false] {
            harness.event(Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        harness.run();
        let second = indicator(&mut harness, &format!("current-line-{theme}-pointer"))
            .expect("clicking the second line moves the indicator");
        assert!((second.top() - first.top() - line_height).abs() <= 1.0);
        harness.key_press_modifiers(Modifiers::COMMAND, Key::P);
        harness.run();
        assert!(indicator(&mut harness, &format!("current-line-{theme}-search")).is_none());
        harness.key_press(Key::Escape);
        harness.run();
        assert_eq!(
            indicator(&mut harness, &format!("current-line-{theme}-returned")),
            Some(second)
        );
        harness.key_press_modifiers(Modifiers::CTRL, Key::Tab);
        harness.run();
        assert!(
            harness.get_by_label("Note content").is_focused(),
            "next note keeps editor focus"
        );
        harness.key_press_modifiers(Modifiers::CTRL | Modifiers::SHIFT, Key::Tab);
        harness.run();
        assert!(
            harness.get_by_label("Note content").is_focused(),
            "previous note keeps editor focus"
        );
        assert_eq!(harness.state().note.id, original);
        assert_eq!(
            indicator(&mut harness, &format!("current-line-{theme}-restored")),
            Some(second)
        );
        assert_eq!(harness.state().note.body, body);
    }
    Ok(())
}

#[test]
fn a_wrapped_logical_line_keeps_one_continuous_gutter_indicator() -> Result<()> {
    let body = format!("First\n{}\nLast", "한글과 ideas 💡 ".repeat(18));
    let (_directory, app) = fixture(&body, 6, "graphite")?;
    let line_height = app.theme.typography.font_size + app.theme.typography.line_spacing;
    let mut harness = fixtures::harness(app, [320.0, 400.0]);
    harness.run();
    let wrapped = indicator(&mut harness, "current-line-wrapped").expect("wrapped current line");
    assert!(wrapped.height() > line_height * 2.0);
    assert!(wrapped.right() < 64.0);
    harness.key_press(Key::ArrowDown);
    harness.run();
    assert_eq!(
        indicator(&mut harness, "current-line-wrapped-moved"),
        Some(wrapped)
    );
    assert_eq!(harness.state().note.body, body);
    Ok(())
}

#[test]
fn empty_last_lines_and_hangul_composition_keep_the_indicator_on_the_caret() -> Result<()> {
    let (_directory, app) = fixture("", 0, "helium")?;
    let line_height = app.theme.typography.font_size + app.theme.typography.line_spacing;
    let mut harness = fixtures::harness(app, [640.0, 400.0]);
    harness.run();
    let empty = indicator(&mut harness, "current-line-empty").expect("empty draft line one");
    harness.event(Event::Text("An idea\n".into()));
    harness.run();
    let last = indicator(&mut harness, "current-line-last-empty").expect("last empty line");
    assert!((last.top() - empty.top() - line_height).abs() <= 1.0);
    for text in ["ㅎ", "하", "한"] {
        harness.event(Event::Ime(ImeEvent::Preedit {
            text: text.into(),
            active_range_chars: None,
        }));
        harness.run();
        assert_eq!(
            indicator(&mut harness, "current-line-composing"),
            Some(last)
        );
    }
    harness.event(Event::Ime(ImeEvent::Commit("한".into())));
    harness.run();
    assert_eq!(harness.state().note.body, "An idea\n한");
    assert_eq!(
        indicator(&mut harness, "current-line-committed"),
        Some(last)
    );
    Ok(())
}
