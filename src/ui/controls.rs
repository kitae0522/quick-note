use crate::appearance::Palette;
use crate::ui::icons;
use crate::ui::motion;
use eframe::egui;
use eframe::egui::Align2;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::Pos2;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::Sense;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

pub const CONTROL_HEIGHT: f32 = 32.0;

pub fn icon_button(ui: &mut Ui, glyph: &str, label: &str, palette: Palette) -> Response {
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(4.0, 4.0);
            ui.add_sized(
                [
                    CONTROL_HEIGHT,
                    ui.available_height().clamp(24.0, CONTROL_HEIGHT),
                ],
                Button::new(()).frame_when_inactive(false).corner_radius(8),
            )
        })
        .inner;
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
    let pressed = motion::transition(
        ui.ctx(),
        response.id.with("press"),
        response.is_pointer_button_down_on(),
    );
    ui.painter().text(
        response.rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        FontId::new(
            16.0 * (1.0 - 0.04 * pressed),
            FontFamily::Name("icons".into()),
        ),
        palette.foreground,
    );
    focus_ring(ui, &response, palette);
    response.on_hover_text(label)
}

pub fn focus_ring(ui: &Ui, response: &Response, palette: Palette) {
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.shrink(1.0),
            7,
            Stroke::new(2.0, palette.accent),
            StrokeKind::Inside,
        );
    }
}

pub fn paint_icon(ui: &Ui, center: Pos2, glyph: &str, color: Color32) {
    ui.painter().text(
        center,
        Align2::CENTER_CENTER,
        glyph,
        FontId::new(15.0, FontFamily::Name("icons".into())),
        color,
    );
}

/// A full-row switch with the same pointer and keyboard activation path.
pub fn toggle_switch(ui: &mut Ui, label: &str, enabled: bool, palette: Palette) -> Response {
    let label = ui.painter().layout(
        label.to_owned(),
        FontId::proportional(14.0),
        palette.foreground,
        (ui.available_width() - 64.0).max(1.0),
    );
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), (label.size().y + 12.0).max(36.0)),
        Sense::click(),
    );
    response
        .widget_info(|| WidgetInfo::selected(Role::Switch, ui.is_enabled(), enabled, label.text()));
    ui.painter().galley(
        rect.left_top() + egui::vec2(0.0, (rect.height() - label.size().y) / 2.0),
        label,
        palette.foreground,
    );
    let progress = motion::transition(ui.ctx(), response.id.with("thumb"), enabled);
    let track = egui::Rect::from_center_size(
        egui::pos2(rect.right() - 20.0, rect.center().y),
        egui::vec2(36.0, 20.0),
    );
    ui.painter().rect_filled(
        track,
        10,
        palette.secondary.lerp_to_gamma(palette.accent, progress),
    );
    ui.painter().circle_filled(
        egui::pos2(
            egui::lerp(track.left() + 10.0..=track.right() - 10.0, progress),
            track.center().y,
        ),
        7.5,
        palette.background,
    );
    focus_ring(ui, &response, palette);
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
    if response.clicked() {
        ui.ctx().request_repaint();
    }
    response
}

/// A choice keeps its check mark and selected state when motion and color are unavailable.
pub fn choice_button(ui: &mut Ui, label: &str, selected: bool, width: f32) -> Response {
    let response = ui.add_sized(
        [width, CONTROL_HEIGHT],
        Button::new(label)
            .right_text(
                RichText::new(if selected { icons::CHECK } else { "" })
                    .font(FontId::new(14.0, FontFamily::Name("icons".into()))),
            )
            .selected(selected),
    );
    response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label));
    response
}
