//! One note tab owns its title, focus navigation and close target.
use super::HeaderAction;
use super::HeaderState;
use super::tab_drag::TabDrag;
use crate::appearance::Palette;
use crate::model::NoteSummary;
use crate::ui::FocusTarget;
use crate::ui::controls;
use crate::ui::icons;
use eframe::egui;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::DragAndDrop;
use eframe::egui::FontFamily;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Margin;
use eframe::egui::Modifiers;
use eframe::egui::RichText;
use eframe::egui::Role;
use eframe::egui::Sense;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;
use eframe::egui::WidgetInfo;

pub(super) struct NoteTab<'note> {
    pub note: &'note NoteSummary,
    pub index: usize,
    pub tabs: &'note [NoteSummary],
    pub width: f32,
    pub height: f32,
    pub selected: bool,
}

impl NoteTab<'_> {
    pub fn show(
        &self,
        ui: &mut Ui,
        state: &mut HeaderState,
        palette: Palette,
        text: &crate::i18n::Messages,
    ) -> Option<HeaderAction> {
        let mut action = None;
        let title_text = crate::ui::note_title(self.note, text);
        ui.scope_builder(
            UiBuilder::new().scope_id(Id::unique(("note-tab", &self.note.id))),
            |ui| {
                let fill = if self.selected {
                    palette.background
                } else {
                    Color32::TRANSPARENT
                };
                let tab = Frame::new()
                    .fill(fill)
                    .corner_radius(8)
                    .inner_margin(Margin::symmetric(2, 0))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal(|ui| {
                            let sense = if self.selected {
                                Sense::click_and_drag()
                            } else {
                                Sense::CLICK | Sense::DRAG
                            };
                            let family = if self.selected {
                                FontFamily::Name("medium".into())
                            } else {
                                FontFamily::Proportional
                            };
                            let mut color = if self.selected {
                                palette.foreground
                            } else {
                                palette.secondary
                            };
                            if TabDrag::is_dragging(ui, &self.note.id) {
                                color = color.gamma_multiply(0.4);
                            }
                            let title = RichText::new(title_text)
                                .font(FontId::new(13.0, family))
                                .color(color);
                            let response = ui.add_sized(
                                [self.width - 28.0, self.height],
                                Button::new(title)
                                    .right_text("")
                                    .truncate()
                                    .frame_when_inactive(false)
                                    .sense(sense),
                            );
                            response.widget_info(|| {
                                WidgetInfo::selected(Role::Tab, true, self.selected, title_text)
                            });
                            TabDrag::start(&response, &self.note.id, title_text, self.width);
                            if state.focus_tab.as_deref() == Some(self.note.id.as_str()) {
                                response.request_focus();
                                state.focus_tab = None;
                            }
                            controls::focus_ring(ui, &response, palette);
                            if response.clicked() && !DragAndDrop::has_any_payload(ui.ctx()) {
                                action = Some(HeaderAction::Select {
                                    id: self.note.id.clone(),
                                    focus: FocusTarget::Editor,
                                });
                            }
                            if response.has_focus() {
                                let next = ui.input_mut(|input| {
                                    if input.consume_key(Modifiers::NONE, Key::ArrowRight) {
                                        Some((self.index + 1) % self.tabs.len())
                                    } else if input.consume_key(Modifiers::NONE, Key::ArrowLeft) {
                                        Some((self.index + self.tabs.len() - 1) % self.tabs.len())
                                    } else if input.consume_key(Modifiers::NONE, Key::Home) {
                                        Some(0)
                                    } else if input.consume_key(Modifiers::NONE, Key::End) {
                                        Some(self.tabs.len() - 1)
                                    } else {
                                        None
                                    }
                                });
                                if let Some(index) = next {
                                    state.focus_tab = Some(self.tabs[index].id.clone());
                                    state.reveal_active = true;
                                    action = Some(HeaderAction::Select {
                                        id: self.tabs[index].id.clone(),
                                        focus: FocusTarget::Tab,
                                    });
                                }
                            }
                            response.on_hover_text(title_text);
                            ui.spacing_mut().button_padding = egui::vec2(4.0, 4.0);
                            let close = ui.add_sized(
                                [24.0, self.height],
                                Button::new(
                                    RichText::new(icons::CLOSE)
                                        .font(FontId::new(12.0, FontFamily::Name("icons".into())))
                                        .color(palette.secondary),
                                )
                                .frame_when_inactive(false)
                                .sense(Sense::CLICK),
                            );
                            close.widget_info(|| {
                                WidgetInfo::labeled(
                                    Role::Button,
                                    true,
                                    crate::i18n::format(
                                        &text.close_named_note,
                                        &[("title", title_text)],
                                    ),
                                )
                            });
                            if close.clicked() {
                                action = Some(HeaderAction::CloseTab(self.note.id.clone()));
                            }
                            close.on_hover_text(crate::i18n::format(
                                &text.close_named_note,
                                &[("title", title_text)],
                            ));
                        });
                    });
                if self.selected {
                    let rect = tab.response.rect;
                    ui.painter().hline(
                        rect.left() + 10.0..=rect.right() - 10.0,
                        rect.bottom() - 1.0,
                        Stroke::new(2.0, palette.accent),
                    );
                }
            },
        );
        action
    }
}
