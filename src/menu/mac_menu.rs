//! Keep AppKit objects on the main thread and wake egui only when a menu is chosen.
use super::MenuAction;
use super::MenuState;
use super::builder::BuiltMenu;
use crate::i18n::Messages;
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
                    .map(|(_, action, _)| *action)
            })
            .collect()
    }

    pub fn sync(&mut self, state: MenuState, text: &Messages) {
        if self.state == Some(state) {
            return;
        }
        for (item, action, hint) in &self.built.items {
            item.set_enabled(action.enabled(state));
            let label = if *action == MenuAction::App(Command::Delete) {
                Some(if state.settings {
                    &text.back_to_note
                } else {
                    &text.shortcut_delete
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
}
