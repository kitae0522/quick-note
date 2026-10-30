//! A small status menu opens captures and the five most recently saved notes.
use super::MenuAction;
use crate::i18n;
use crate::i18n::Messages;
use crate::model::NoteSummary;
use crate::shortcut::Command;
use crate::storage::Result;
use muda::Menu;
use muda::MenuId;
use muda::MenuItem;
use muda::PredefinedMenuItem;
use muda::Submenu;
use tray_icon::Icon;
use tray_icon::TrayIcon;
use tray_icon::TrayIconBuilder;

pub(super) struct ResidentMenu {
    pub tray: TrayIcon,
    #[cfg(feature = "diagnostics")]
    pub native: Option<objc2::rc::Retained<objc2_app_kit::NSMenu>>,
    items: Vec<(MenuItem, MenuAction)>,
    notes: Vec<NoteSummary>,
}

impl ResidentMenu {
    pub fn new(text: &Messages, notes: Vec<NoteSummary>) -> Result<Self> {
        let tray = TrayIconBuilder::new()
            .with_tooltip(&text.app_name)
            .with_icon_templated(icon()?)
            .build()?;
        let mut resident = Self {
            tray,
            #[cfg(feature = "diagnostics")]
            native: None,
            items: Vec::new(),
            notes: Vec::new(),
        };
        resident.rebuild(text, notes)?;
        Ok(resident)
    }

    pub fn refresh(&mut self, text: &Messages, notes: Vec<NoteSummary>) -> Result<()> {
        if self.notes != notes {
            self.rebuild(text, notes)?;
        }
        Ok(())
    }

    pub fn relabel(&mut self, text: &Messages) -> Result<()> {
        self.rebuild(text, self.notes.clone())
    }

    fn rebuild(&mut self, text: &Messages, notes: Vec<NoteSummary>) -> Result<()> {
        let menu = Menu::new();
        let mut items = Vec::new();
        for (label, command) in [
            (&text.capture.new_note, Command::NewCapture),
            (&text.quick_capture, Command::Capture),
            (&text.capture.clipboard, Command::CaptureClipboard),
        ] {
            let item = MenuItem::new(label, true, None);
            menu.append(&item)?;
            items.push((item, MenuAction::App(command)));
        }
        menu.append(&PredefinedMenuItem::separator())?;
        let recent = Submenu::new(&text.capture.recent_notes, !notes.is_empty());
        for note in &notes {
            let title = if note.is_blank {
                text.note_untitled.as_str()
            } else {
                &note.title
            };
            let item = MenuItem::new(title, true, None);
            recent.append(&item)?;
            items.push((item, MenuAction::OpenNote(note.id.clone())));
        }
        menu.append(&recent)?;
        menu.append(&PredefinedMenuItem::separator())?;
        for (label, command) in [
            (text.menu_settings.clone(), Command::Options),
            (
                i18n::format(&text.menu_quit, &[("app", &text.app_name)]),
                Command::Quit,
            ),
        ] {
            let item = MenuItem::new(label, true, None);
            menu.append(&item)?;
            items.push((item, MenuAction::App(command)));
        }
        #[cfg(feature = "diagnostics")]
        {
            // tray-icon attaches its NSMenu only while tracking a popup. Obtain a safe retained
            // handle through AppKit, restoring the main menu before the event loop can run.
            let main = objc2::MainThreadMarker::new().expect("status menus use the main thread");
            let app = objc2_app_kit::NSApplication::sharedApplication(main);
            let previous = app.mainMenu();
            menu.init_for_nsapp();
            self.native = app.mainMenu();
            app.setMainMenu(previous.as_deref());
        }
        self.tray.set_menu(Some(Box::new(menu)));
        self.items = items;
        self.notes = notes;
        Ok(())
    }

    pub fn action(&self, id: &MenuId) -> Option<MenuAction> {
        self.items
            .iter()
            .find(|(item, _)| item.id() == id)
            .map(|(_, action)| action.clone())
    }
}

fn icon() -> Result<Icon> {
    let mut rgba = vec![0; 18 * 18 * 4];
    for y in 2..16 {
        for x in 3..15 {
            let outline = x == 3 || x == 14 || y == 2 || y == 15;
            let line = (6..=11).contains(&x) && [6, 9, 12].contains(&y);
            if outline || line {
                rgba[(y * 18 + x) * 4 + 3] = 255;
            }
        }
    }
    Ok(Icon::from_rgba(rgba, 18, 18)?)
}
