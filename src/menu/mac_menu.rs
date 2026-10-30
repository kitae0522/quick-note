//! Keep AppKit objects on the main thread and wake egui only when a menu is chosen.
use super::MenuAction;
use super::MenuState;
use super::builder::BuiltMenu;
use super::resident_menu::ResidentMenu;
use crate::i18n::Messages;
use crate::model::NoteSummary;
use crate::shortcut::Command;
use crate::shortcut::Shortcuts;
use crate::storage::Result;
use eframe::egui::Context;
use muda::MenuEvent;
use objc2::MainThreadMarker;
use objc2_app_kit::NSApplication;
use std::sync::mpsc;

pub(crate) struct MacMenu {
    built: BuiltMenu,
    receiver: mpsc::Receiver<MenuEvent>,
    state: Option<MenuState>,
    resident: Option<ResidentMenu>,
}

impl MacMenu {
    pub fn visibility(action: MenuAction) {
        let main = MainThreadMarker::new().expect("native menus execute on the AppKit main thread");
        let app = NSApplication::sharedApplication(main);
        match action {
            MenuAction::Hide => app.hide(None),
            MenuAction::HideOthers => app.hideOtherApplications(None),
            MenuAction::ShowAll => app.unhideAllApplications(None),
            _ => {}
        }
    }

    pub fn new(text: &Messages, shortcuts: &Shortcuts, context: &Context) -> Result<Self> {
        let built = BuiltMenu::new(text, shortcuts, context)?;
        let (sender, receiver) = mpsc::channel();
        let context = context.clone();
        MenuEvent::set_event_handler(Some(move |event| {
            if sender.send(event).is_ok() {
                context.request_repaint();
            }
        }));
        built.root.init_for_nsapp();
        Ok(Self {
            built,
            receiver,
            state: None,
            resident: None,
        })
    }

    pub fn rebuild(
        &mut self,
        text: &Messages,
        shortcuts: &Shortcuts,
        context: &Context,
    ) -> Result<()> {
        let built = BuiltMenu::new(text, shortcuts, context)?;
        built.root.init_for_nsapp();
        self.built = built;
        self.state = None;
        if let Some(resident) = &mut self.resident {
            resident.relabel(text)?;
        }
        Ok(())
    }

    pub fn receive(&self) -> Vec<MenuAction> {
        self.receiver
            .try_iter()
            .filter_map(|event| {
                self.built
                    .items
                    .iter()
                    .find(|(item, _, _)| item.id() == &event.id)
                    .map(|(_, action, _)| action.clone())
                    .or_else(|| {
                        self.resident
                            .as_ref()
                            .and_then(|resident| resident.action(&event.id))
                    })
            })
            .collect()
    }

    pub fn sync(&mut self, state: MenuState, text: &Messages) {
        if self.state == Some(state) {
            return;
        }
        for (item, action, hint) in &self.built.items {
            item.set_enabled(action.enabled(state));
            let label = if *action == MenuAction::App(Command::CloseTab) {
                Some(if state.settings {
                    &text.back_to_note
                } else {
                    &text.close_note
                })
            } else if *action == MenuAction::App(Command::Preview) {
                Some(if state.preview {
                    &text.edit
                } else {
                    &text.preview
                })
            } else {
                None
            };
            if let Some(label) = label {
                item.set_text(
                    hint.as_ref()
                        .map_or_else(|| label.clone(), |hint| format!("{label} ({hint})")),
                );
            }
        }
        self.state = Some(state);
    }
    pub fn install_resident(&mut self, text: &Messages, notes: Vec<NoteSummary>) -> Result<()> {
        if self.resident.is_none() {
            self.resident = Some(ResidentMenu::new(text, notes)?);
        }
        Ok(())
    }

    pub fn resident_installed(&self) -> bool {
        self.resident.is_some()
    }

    pub fn refresh_recent(&mut self, text: &Messages, notes: Vec<NoteSummary>) -> Result<()> {
        if let Some(resident) = &mut self.resident {
            resident.refresh(text, notes)?;
        }
        Ok(())
    }

    pub fn capture_feedback(&self, message: Option<&str>) {
        if let Some(resident) = &self.resident {
            resident.tray.set_title(message.map(|_| "✓"));
            let _ = resident.tray.set_tooltip(message);
        }
    }

    #[cfg(feature = "diagnostics")]
    pub fn resident_menu(&self) -> Option<objc2::rc::Retained<objc2_app_kit::NSMenu>> {
        self.resident.as_ref()?.native.clone()
    }
}
