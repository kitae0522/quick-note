use crate::appearance::Theme;
use crate::model::CursorLocation;
use crate::model::Note;
use crate::ui::EditorMetrics;
use crate::ui::editor_find;
use crate::ui::editor_find::FindHighlights;
use crate::ui::editor_gutter;
use crate::ui::editor_pairs;
use eframe::egui;
use eframe::egui::Align2;
use eframe::egui::Event;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Margin;
use eframe::egui::ScrollArea;
use eframe::egui::TextBuffer;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

#[must_use]
pub struct EditorOutput {
    pub changed: bool,
    pub selection_changed: bool,
    pub location: CursorLocation,
}

pub(crate) struct EditorInteraction<'a> {
    pub quiet: bool,
    pub focus: &'a mut bool,
    pub restore_cursor: &'a mut bool,
    pub find: Option<FindHighlights<'a>>,
}

pub(crate) fn editor(
    ui: &mut Ui,
    note: &mut Note,
    metrics: &mut EditorMetrics,
    theme: &Theme,
    interaction: EditorInteraction<'_>,
    text: &crate::i18n::Messages,
) -> EditorOutput {
    let EditorInteraction {
        quiet,
        focus,
        restore_cursor,
        find,
    } = interaction;
    let palette = theme.palette();
    let editor_id = Id::unique(("note-editor", &note.id));
    let canvas = ui.available_rect_before_wrap();
    metrics.update(note, *restore_cursor);
    // Focus before TextEdit reads this frame's first character.
    if *focus {
        ui.memory_mut(|memory| memory.request_focus(editor_id));
    }
    if *restore_cursor {
        let (start, end) = metrics.lines().char_range(&note.body, &note.selection);
        let mut state =
            egui::text_edit::TextEditState::load(ui.ctx(), editor_id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(start),
                egui::text::CCursor::new(end),
            )));
        state.store(ui.ctx(), editor_id);
        *restore_cursor = false;
    }
    if ui.memory(|memory| memory.has_focus(editor_id)) {
        // Keep indentation in TextEdit's input path so selection, IME and undo stay together.
        ui.input_mut(|input| {
            for event in &mut input.events {
                if let Event::Key {
                    key: Key::Tab,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                    && modifiers.is_none()
                {
                    *event = Event::Text("    ".into());
                }
            }
        });
    }
    let before = note.selection.clone();
    let mut result = EditorOutput {
        changed: false,
        selection_changed: false,
        location: CursorLocation { line: 1, column: 1 },
    };
    let padding = theme.layout.editor_padding;
    ScrollArea::vertical()
        .id_salt(("note-source", &note.id))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let digit_count = metrics.lines().starts.len().ilog10() + 1;
            let gutter = if quiet {
                padding
            } else {
                (digit_count as f32 * 7.5 + 12.0).max(24.0) + padding
            };
            let width = ui.available_width();
            let font_id = crate::appearance::editor_font(theme.typography.font_size);
            let line_height = theme.typography.font_size + theme.typography.line_spacing;
            let mut layouter = |ui: &Ui, buffer: &dyn TextBuffer, wrap_width: f32| {
                let mut job = egui::text::LayoutJob::simple(
                    buffer.as_str().to_owned(),
                    font_id.clone(),
                    palette.foreground,
                    wrap_width,
                );
                for section in &mut job.sections {
                    section.format.line_height = Some(line_height);
                }
                ui.fonts_mut(|fonts| fonts.layout_job(job))
            };
            let highlights = find.as_ref().map(|_| ui.painter().add(egui::Shape::Noop));
            let mut pair_input = editor_pairs::input;
            let output = TextEdit::multiline(&mut note.body)
                .input_handler(&mut pair_input)
                .id(editor_id)
                .font(font_id.clone())
                .frame(Frame::NONE.inner_margin(Margin {
                    left: gutter.round() as i8,
                    right: padding.round() as i8,
                    top: padding.round() as i8,
                    bottom: padding.round() as i8,
                }))
                .desired_width(width)
                .desired_rows(1)
                .min_size(egui::vec2(width, canvas.height()))
                .align(Align2::LEFT_TOP)
                .lock_focus(true)
                .text_color(palette.foreground)
                .layouter(&mut layouter)
                .show(ui);
            if *focus {
                // A click on New note or a closing modal can blur the editor during layout.
                output.response.request_focus();
                *focus = !output.response.has_focus();
            }
            output.response.widget_info(|| {
                let mut info = WidgetInfo::text_edit(true, &note.body, &note.body, "");
                info.label = Some(text.note_content.as_str().into());
                info
            });
            result.changed = output.response.changed();
            if !result.changed
                && let (Some(find), Some(layer)) = (&find, highlights)
            {
                editor_find::paint(ui, &output, find, palette, layer);
            }
            metrics.update(note, result.changed);
            let lines = metrics.lines();
            let caret = if let Some(cursor) = output.cursor_range {
                let range = cursor.as_sorted_char_range();
                note.selection = lines.selection(&note.body, range.start.into(), range.end.into());
                cursor.primary.index.into()
            } else {
                lines.char_range(&note.body, &note.selection).1
            };
            result.location = lines.location(&note.body, caret);
            result.selection_changed = note.selection != before;
            if !quiet {
                editor_gutter::paint(ui, &output, result.location.line, palette);
            }
        });
    result
}
