use crate::fixtures;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::Shape;
use quick_note::storage::Result;

#[cfg(target_os = "macos")]
#[test]
fn find_highlights_preserve_the_kerning_of_mixed_script_text() -> Result<()> {
    let (_directory, mut app) = fixtures::fixture()?;
    let body = "A thought, clearly set.\n\nAVATAR To. Writing feels immediate.\n한글 자간과 영문 간격을 함께 확인해요.\nQuick Note에서 아이디어를 기록해요.\n\n한글과 English, 숫자 0123456789, 그리고 💡\n문장을 찾고, 고치고, 계속 써요.\n";
    app.note.body = body.into();
    app.dirty = true;
    assert!(app.persist());
    app.set_editor_fonts(
        Some(&[
            "SF Pro".into(),
            "Pretendard".into(),
            "monospace".into(),
            "Noto Sans CJK".into(),
        ]),
        &Context::default(),
    )?;
    let mut harness = fixtures::harness(app, [920.0, 600.0]);
    harness.run();
    fixtures::capture(&mut harness, "typography-editor");
    let positions = |shapes: &[eframe::egui::epaint::ClippedShape]| {
        shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                Shape::Text(text) if text.galley.job.text == body => Some(
                    text.galley
                        .rows
                        .iter()
                        .flat_map(|row| {
                            row.glyphs
                                .iter()
                                .map(|glyph| (glyph.pos, glyph.advance_width))
                        })
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .expect("the source editor paints its full note")
    };
    let before = positions(&harness.output().shapes);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    harness.event(Event::Text("To".into()));
    harness.run();
    assert_eq!(positions(&harness.output().shapes), before);
    fixtures::capture(&mut harness, "typography-find");
    Ok(())
}
