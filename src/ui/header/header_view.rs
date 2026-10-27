//! The shared one-row layout composes search, tabs and native controls.
use super::HeaderAction;
use super::HeaderOutput;
use super::HeaderState;
use super::WindowControls;
use super::search_field;
use super::tab_strip::TabStrip;
use super::window_drag;
use crate::appearance::Palette;
use crate::model::NoteSummary;
use crate::ui::controls;
use crate::ui::icons;
use crate::ui::safe_area;
use eframe::egui;
use eframe::egui::Align;
use eframe::egui::Frame;
use eframe::egui::Layout;
use eframe::egui::Margin;
use eframe::egui::Rect;
use eframe::egui::Role;
use eframe::egui::Sense;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::WidgetInfo;

pub fn header(
    ui: &mut Ui,
    state: &mut HeaderState,
    tabs: &[NoteSummary],
    active: &str,
    palette: Palette,
    platform: WindowControls,
    text: &crate::i18n::Messages,
) -> HeaderOutput {
    let width = ui.available_width();
    let height = ui.available_height().clamp(32.0, 56.0);
    let left = ui.max_rect().left()
        + if platform == WindowControls::Mac {
            safe_area::MAC_CONTROLS_WIDTH as f32
        } else {
            0.0
        };
    let margin = Rect::from_min_size(
        egui::pos2(left, ui.max_rect().top()),
        egui::vec2(safe_area::EDGE_INSET as f32, height),
    );
    window_drag::hand_off(ui.interact(
        margin,
        ui.scope_id().with("window-drag-margin"),
        Sense::DRAG,
    ));
    let mut output = HeaderOutput {
        action: None,
        search_rect: Rect::NOTHING,
        search_changed: false,
        search_gained_focus: false,
    };
    Frame::new()
        .fill(palette.toolbar)
        .inner_margin(Margin {
            left: safe_area::EDGE_INSET
                + if platform == WindowControls::Mac {
                    safe_area::MAC_CONTROLS_WIDTH
                } else {
                    0
                },
            right: safe_area::EDGE_INSET,
            top: 2,
            bottom: 2,
        })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(height - 4.0);
            ui.set_max_height(height - 4.0);
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let control_height = ui.available_height().clamp(24.0, controls::CONTROL_HEIGHT);
                ui.spacing_mut().interact_size.y = control_height;
                output = search_field::show(ui, state, width, palette, text);
                let window_width = if platform == WindowControls::Windows {
                    108.0
                } else {
                    0.0
                };
                let tab_space = (ui.available_width() - 76.0 - window_width).max(32.0);
                let desired_tab = if tabs.len() <= 4 {
                    ((tab_space - 2.0 * tabs.len().saturating_sub(1) as f32)
                        / tabs.len().max(1) as f32)
                        .clamp(96.0, 188.0)
                } else {
                    144.0
                };
                let desired_total = (desired_tab + 2.0) * tabs.len() as f32 - 2.0;
                let overflow = desired_total > tab_space;
                let scroll_buttons = overflow && width >= 480.0;
                let scroll_width = if overflow {
                    (tab_space - if scroll_buttons { 72.0 } else { 0.0 }).max(30.0)
                } else {
                    desired_total.max(0.0)
                };
                if scroll_buttons
                    && controls::icon_button(
                        ui,
                        icons::LEFT,
                        text.scroll_tabs_left.as_str(),
                        palette,
                    )
                    .clicked()
                {
                    state.scroll = (state.scroll - 200.0).max(0.0);
                }
                ui.allocate_ui_with_layout(
                    egui::vec2(scroll_width, control_height),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                        let strip = TabStrip {
                            tabs,
                            active,
                            width: scroll_width,
                            tab_width: desired_tab.min(scroll_width),
                            height: control_height,
                            palette,
                            text,
                        };
                        if let Some(action) = strip.show(ui, state) {
                            output.action = Some(action);
                        }
                    },
                )
                .response
                .widget_info(|| WidgetInfo::labeled(Role::TabList, true, text.notes.as_str()));
                if scroll_buttons
                    && controls::icon_button(
                        ui,
                        icons::RIGHT,
                        text.scroll_tabs_right.as_str(),
                        palette,
                    )
                    .clicked()
                {
                    state.scroll += 200.0;
                }
                if controls::icon_button(ui, icons::PLUS, text.new_note.as_str(), palette).clicked()
                {
                    output.action = Some(HeaderAction::New);
                }
                let spacer = ui.available_width() - 36.0 - window_width;
                if spacer > 0.0 {
                    window_drag::hand_off(
                        ui.allocate_response(egui::vec2(spacer, control_height), Sense::DRAG),
                    );
                }
                if controls::icon_button(ui, icons::MORE, text.more_options.as_str(), palette)
                    .clicked()
                {
                    output.action = Some(HeaderAction::Options);
                }
                if platform == WindowControls::Windows {
                    for (icon, label, action) in [
                        (
                            icons::MINUS,
                            text.minimize_window.as_str(),
                            HeaderAction::Minimize,
                        ),
                        (
                            icons::SQUARE,
                            text.maximize_window.as_str(),
                            HeaderAction::Maximize,
                        ),
                        (
                            icons::CLOSE,
                            text.close_window.as_str(),
                            HeaderAction::CloseWindow,
                        ),
                    ] {
                        if controls::icon_button(ui, icon, label, palette).clicked() {
                            output.action = Some(action);
                        }
                    }
                }
            });
        });
    ui.painter().hline(
        ui.min_rect().x_range(),
        ui.min_rect().bottom(),
        Stroke::new(1.0, palette.divider),
    );
    output
}
