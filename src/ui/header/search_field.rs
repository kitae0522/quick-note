//! Compact search input and icon fallback.
use super::HeaderAction;
use super::HeaderOutput;
use super::HeaderState;
use crate::appearance::Palette;
use crate::ui::controls;
use crate::ui::icons;
use eframe::egui;
use eframe::egui::FontId;
use eframe::egui::Margin;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

pub(super) fn show(
    ui: &mut Ui,
    state: &mut HeaderState,
    width: f32,
    palette: Palette,
    text: &crate::i18n::Messages,
) -> HeaderOutput {
    let mut action = None;
    let search_rect;
    let mut search_changed = false;
    let mut search_focused = false;
    let search_width = if width >= 800.0 {
        186.0
    } else if width >= 620.0 {
        144.0
    } else {
        32.0
    };
    if search_width > 32.0 {
        let id = ui.make_persistent_id("search-notes");
        if state.focus_search {
            ui.memory_mut(|memory| memory.request_focus(id));
        }
        let previous = state.query.clone();
        let response = ui.add_sized(
            [
                search_width,
                ui.available_height().clamp(24.0, controls::CONTROL_HEIGHT),
            ],
            TextEdit::singleline(&mut state.query)
                .return_key(None)
                .id(id)
                .hint_text(text.search_notes.as_str())
                .font(FontId::proportional(13.0))
                .prefix(
                    egui::RichText::new(icons::SEARCH)
                        .font(FontId::new(15.0, egui::FontFamily::Name("icons".into())))
                        .color(palette.secondary),
                )
                .frame(
                    egui::Frame::new()
                        .fill(palette.background)
                        .corner_radius(8)
                        .inner_margin(Margin::symmetric(10, 0)),
                )
                .align(egui::Align2::LEFT_CENTER),
        );
        if state.focus_search {
            response.request_focus();
            state.focus_search = !response.has_focus();
        }
        response.widget_info(|| {
            let mut info =
                WidgetInfo::text_edit(true, &previous, &state.query, text.search_notes.as_str());
            info.label = Some(text.search_notes.as_str().into());
            info
        });
        search_changed = response.changed();
        search_focused = response.has_focus();
        controls::focus_ring(ui, &response, palette);
        search_rect = response.rect;
    } else {
        let response = controls::icon_button(ui, icons::SEARCH, text.find_note.as_str(), palette);
        search_rect = response.rect;
        if response.clicked() || state.focus_search {
            action = Some(HeaderAction::Search);
            state.focus_search = false;
        }
    }

    HeaderOutput {
        action,
        search_rect,
        search_changed,
        search_focused,
    }
}
