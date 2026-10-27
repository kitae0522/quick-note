use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Shape;
use eframe::egui::epaint::TextShape;
use egui_kittest::Harness;
use quick_note::app::Mode;
use quick_note::app::QuickNoteApp;
use quick_note::storage::Result;

fn painted_text<'a>(harness: &'a Harness<'_, QuickNoteApp>, value: &str) -> &'a TextShape {
    harness
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            Shape::Text(text) if text.galley.job.text == value => Some(text),
            _ => None,
        })
        .expect("the Markdown item paints its text")
}

fn baseline(text: &TextShape) -> f32 {
    let row = &text.galley.rows[0];
    text.pos.y + row.pos.y + row.glyphs[0].pos.y
}

#[test]
fn list_markers_align_with_the_first_text_line_in_both_themes_and_widths() -> Result<()> {
    let items = ["Alpha item", "한글 항목", "Nested item", "Ordered item"];
    for theme in ["helium", "graphite"] {
        for width in [920.0, 320.0] {
            let (_directory, mut app) = fixtures::fixture()?;
            app.set_theme(theme, &Context::default())?;
            app.note.body =
                "- Alpha item\n- 한글 항목\n  - Nested item\n\n1. Ordered item\n".into();
            app.mode = Mode::Preview;
            let body = app.note.body.clone();
            let mut harness = fixtures::harness(app, [width, 600.0]);
            harness.run();
            if theme == "helium" && width == 920.0 {
                fixtures::capture(&mut harness, "markdown-list-alignment");
            }
            let markers: Vec<_> = harness
                .output()
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    Shape::Circle(circle) if circle.radius < 6.0 => Some(circle.center),
                    _ => None,
                })
                .collect();
            assert_eq!(markers.len(), 3);
            for item in &items[..3] {
                let text = painted_text(&harness, item);
                let glyph = &text.galley.rows[0].glyphs[0];
                let center = baseline(text) - glyph.font_ascent + glyph.font_height / 2.0;
                let marker = markers
                    .iter()
                    .min_by(|a, b| (a.y - center).abs().total_cmp(&(b.y - center).abs()))
                    .expect("three list markers");
                assert!(
                    (marker.y - center).abs() <= 1.0,
                    "{item}: {marker:?}, text center={center}"
                );
            }
            let number = painted_text(&harness, "1.");
            let text = painted_text(&harness, items[3]);
            assert!((baseline(number) - baseline(text)).abs() <= 1.0);
            assert_eq!(harness.state().note.body, body);
        }
    }
    Ok(())
}

#[test]
fn wrapped_and_nested_lists_keep_their_text_indentation() -> Result<()> {
    let parent = "A parent item that wraps onto multiple lines in the narrow preview pane.";
    let nested = "중첩 목록도 여러 줄로 내려갈 때 본문의 시작 위치를 유지해야 해요.";
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body = format!("- {parent}\n  - {nested}\n");
    app.mode = Mode::Preview;
    let mut harness = fixtures::harness(app, [320.0, 600.0]);
    harness.run();
    fixtures::capture(&mut harness, "markdown-list-wrapped");
    let mut leading_edges = Vec::new();
    for item in [parent, nested] {
        let text = painted_text(&harness, item);
        assert!(text.galley.rows.len() > 1);
        let edges: Vec<_> = text
            .galley
            .rows
            .iter()
            .filter_map(|row| {
                row.glyphs
                    .first()
                    .map(|glyph| text.pos.x + row.pos.x + glyph.pos.x)
            })
            .collect();
        assert!(
            edges.iter().all(|x| (*x - edges[0]).abs() <= 1.0),
            "{item}: {edges:?}"
        );
        leading_edges.push(edges[0]);
    }
    assert!(leading_edges[1] > leading_edges[0] + 8.0);
    Ok(())
}

#[test]
fn resized_cached_lists_keep_nested_text_within_the_content_margin() -> Result<()> {
    let nested = "중첩된 목록도 첫 줄과 표시를 맞춰요";
    let (_directory, mut app) = fixtures::fixture()?;
    app.note.body =
        format!("## List alignment\n\n- Alpha item\n- 이런 게 위치가 안 맞아요\n  - {nested}\n");
    app.mode = Mode::Preview;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    for size in [[320.0, 480.0], [920.0, 600.0]] {
        harness.set_size(size.into());
        harness.run();
        let text = painted_text(&harness, nested);
        for row in &text.galley.rows {
            let rect = row
                .rect_without_leading_space()
                .translate(text.pos.to_vec2());
            assert!(rect.right() <= size[0] - 24.0 + 1.0, "{rect:?}");
        }
    }
    Ok(())
}
