//! Keyboard help displays the resolved runtime bindings rather than a second default table.
use crate::i18n::Messages;
use crate::shortcut::ACTIONS;
use crate::shortcut::Command;
use crate::shortcut::Shortcuts;
use eframe::egui::Grid;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::Ui;

pub(super) fn show_help(
    ui: &mut Ui,
    shortcuts: &Shortcuts,
    text: &Messages,
    markdown_enabled: bool,
) {
    ui.spacing_mut().item_spacing.y = 8.0;
    Grid::new("configured-shortcuts")
        .num_columns(2)
        .spacing(eframe::egui::vec2(16.0, 8.0))
        .max_col_width(((ui.available_width() - 16.0) / 2.0).max(0.0))
        .show(ui, |ui| {
            for action in ACTIONS {
                if action.command == Command::Preview && !markdown_enabled {
                    continue;
                }
                let label = (action.label)(text);
                let label = if let Command::SelectNote(index) = action.command
                    && index != usize::MAX
                {
                    crate::i18n::format(label, &[("number", &(index + 1).to_string())])
                } else {
                    label.to_owned()
                };
                let keys = shortcuts.key_text(action.command, ui.ctx());
                let keys = if keys.is_empty() {
                    text.shortcut_unassigned.clone()
                } else {
                    keys
                };
                ui.add(Label::new(label).wrap());
                ui.add(Label::new(RichText::new(keys).weak()).wrap());
                ui.end_row();
            }
        });
    ui.separator();
    for help in [
        &text.help_tab,
        &text.help_tabs,
        &text.help_activate,
        &text.help_escape,
        &text.help_editing,
    ] {
        ui.label(help.as_str());
    }
}
