//! Apply tokens to native widget visuals without changing cursor preferences.
use super::Theme;
use eframe::egui;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::CornerRadius;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::Stroke;
use eframe::egui::TextStyle;
use eframe::egui::Theme as GuiTheme;
use eframe::egui::Visuals;
use eframe::egui::epaint::Shadow;

impl Theme {
    /// Applies a validated theme while retaining the existing caret preference.
    ///
    /// # Panics
    /// Panics if color tokens were changed without validation.
    pub fn apply(&self, context: &Context) {
        let palette = self.palette();
        let cursor = context
            .style_of(context.theme())
            .visuals
            .text_cursor
            .clone();
        context.set_theme(if self.appearance == "dark" {
            GuiTheme::Dark
        } else {
            GuiTheme::Light
        });
        let mut visuals = if self.appearance == "dark" {
            Visuals::dark()
        } else {
            Visuals::light()
        };
        visuals.text_cursor = cursor;
        visuals.panel_fill = palette.background;
        visuals.window_fill = palette.background;
        visuals.extreme_bg_color = palette.background;
        visuals.faint_bg_color = palette.toolbar;
        visuals.override_text_color = Some(palette.foreground);
        visuals.weak_text_color = Some(palette.secondary);
        visuals.hyperlink_color = palette.accent;
        visuals.selection.bg_fill = palette.selection;
        visuals.selection.stroke = Stroke::new(1.0, palette.foreground);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.foreground);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.divider);
        visuals.widgets.inactive.bg_fill = palette.toolbar;
        visuals.widgets.inactive.weak_bg_fill = palette.toolbar;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.divider);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.foreground);
        visuals.widgets.hovered.bg_fill = palette.selection;
        visuals.widgets.hovered.weak_bg_fill = palette.selection;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.foreground);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.divider);
        visuals.widgets.active.bg_fill = palette.selection;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.foreground);
        visuals.widgets.active.bg_stroke = Stroke::new(2.0, palette.accent);
        visuals.window_stroke = Stroke::new(1.0, palette.divider);
        visuals.window_corner_radius = CornerRadius::same(16);
        visuals.menu_corner_radius = CornerRadius::same(8);
        visuals.window_highlight_topmost = false;
        visuals.window_shadow = Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(if visuals.dark_mode { 80 } else { 24 }),
        };
        visuals.popup_shadow = Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(if visuals.dark_mode { 64 } else { 18 }),
        };
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.corner_radius = CornerRadius::same(8);
        }
        context.set_visuals(visuals);
        // A palette swap is immediate, without carrying the previous theme's transitions.
        context.clear_animations();
        context.all_styles_mut(|style| {
            style.spacing.item_spacing = egui::vec2(6.0, 8.0);
            style.spacing.button_padding = egui::vec2(10.0, 6.0);
            style.spacing.interact_size = egui::vec2(28.0, 30.0);
            style
                .text_styles
                .insert(TextStyle::Body, FontId::proportional(14.0));
            style
                .text_styles
                .insert(TextStyle::Button, FontId::proportional(13.0));
            style
                .text_styles
                .insert(TextStyle::Small, FontId::proportional(12.0));
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::monospace(self.typography.font_size),
            );
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(26.0, FontFamily::Name("strong".into())),
            );
        });
    }
}
