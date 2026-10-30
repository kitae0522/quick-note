//! Visible search rows show matching text without building widgets for the entire library.
use crate::appearance::Palette;
use crate::i18n;
use crate::i18n::Messages;
use crate::model::NoteSummary;
use crate::model::SearchPreview;
use eframe::egui;
use eframe::egui::Button;
use eframe::egui::FontId;
use eframe::egui::Label;
use eframe::egui::Rect;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::Sense;
use eframe::egui::TextFormat;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;
use eframe::egui::text::LayoutJob;

pub(crate) struct SearchRow<'a> {
    pub note: &'a NoteSummary,
    pub preview: Option<&'a SearchPreview>,
    pub selected: bool,
    pub deleted: bool,
    pub text: &'a Messages,
    pub palette: Palette,
    pub height: f32,
}

impl SearchRow<'_> {
    pub fn show(self, ui: &mut Ui) -> Response {
        let Self {
            note,
            preview,
            selected,
            deleted,
            text,
            palette,
            height,
        } = self;
        let response = ui.add_sized(
            [ui.available_width(), height],
            Button::new("")
                .selected(selected)
                .frame_when_inactive(selected),
        );
        let title = super::note_title(note, text);
        response.widget_info(|| {
            WidgetInfo::selected(
                Role::Button,
                true,
                selected,
                i18n::format(
                    if deleted {
                        &text.restore_note
                    } else {
                        &text.open_note
                    },
                    &[("title", title)],
                ),
            )
        });
        let left = response.rect.left() + 10.0;
        let width = (response.rect.width() - 20.0).max(0.0);
        let line = |y, height| {
            Rect::from_min_size(
                egui::pos2(left, response.rect.top() + y),
                egui::vec2(width, height),
            )
        };
        ui.put(
            line(4.0, 18.0),
            Label::new(RichText::new(title).size(13.0))
                .halign(egui::Align::Min)
                .truncate()
                .sense(Sense::hover()),
        );
        if height >= 40.0
            && let Some(preview) = preview
        {
            let normal = TextFormat {
                font_id: FontId::proportional(12.0),
                color: palette.secondary,
                ..Default::default()
            };
            let mut job = LayoutJob::default();
            if let Some(range) = &preview.highlight {
                job.append(&preview.text[..range.start], 0.0, normal.clone());
                job.append(
                    &preview.text[range.clone()],
                    0.0,
                    TextFormat {
                        color: palette.foreground,
                        background: palette.accent.gamma_multiply(0.25),
                        ..normal.clone()
                    },
                );
                job.append(&preview.text[range.end..], 0.0, normal);
            } else {
                job.append(&preview.text, 0.0, normal);
            }
            ui.put(
                line(23.0, 17.0),
                Label::new(job)
                    .halign(egui::Align::Min)
                    .truncate()
                    .sense(Sense::hover()),
            );
        }
        if height >= 60.0 {
            let elapsed = (crate::storage::timestamp() - note.updated_at).max(0.0) as u64;
            let (template, count) = match elapsed {
                0..60 => (&text.edited_now, 0),
                60..3600 => (&text.edited_minutes, elapsed / 60),
                3600..86400 => (&text.edited_hours, elapsed / 3600),
                _ => (&text.edited_days, elapsed / 86400),
            };
            ui.put(
                line(43.0, 16.0),
                Label::new(
                    RichText::new(i18n::format(template, &[("count", &count.to_string())]))
                        .size(11.0)
                        .color(palette.secondary),
                )
                .halign(egui::Align::Min)
                .truncate()
                .sense(Sense::hover()),
            );
        }
        super::focus_ring(ui, &response, palette);
        response
    }
}
