//! Prepare saved native fonts while the window and GPU initialize.
use super::font_definitions;
use super::system_fonts::SystemFonts;
use crate::storage::Result;
use eframe::egui::Context;
use eframe::egui::FontDefinitions;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

struct PreparedFonts {
    native: SystemFonts,
    initial: Option<(Vec<String>, Result<FontDefinitions>)>,
}

#[derive(Default)]
pub struct FontCatalog {
    pending: Option<mpsc::Receiver<PreparedFonts>>,
    ready: Option<SystemFonts>,
    initial: Option<(Vec<String>, Result<FontDefinitions>)>,
    wake: Arc<Mutex<Option<Context>>>,
}

impl FontCatalog {
    /// Starts preparing the saved stack without requiring a window or egui context.
    pub fn preload(&mut self, names: &[String]) -> Result<()> {
        self.start(Some(names.to_vec()))
    }

    fn start(&mut self, name: Option<Vec<String>>) -> Result<()> {
        if self.ready.is_none() && self.pending.is_none() {
            let (sender, receiver) = mpsc::channel();
            let wake = self.wake.clone();
            thread::Builder::new()
                .name("quick-note-fonts".into())
                .spawn(move || {
                    let mut native = SystemFonts::new();
                    let initial = name.map(|name| {
                        let definitions = font_definitions::definitions(&mut native, &name);
                        (name, definitions)
                    });
                    let _ = sender.send(PreparedFonts { native, initial });
                    let context = wake
                        .lock()
                        .expect("font wake-up only stores a cloned context")
                        .clone();
                    if let Some(context) = context {
                        context.request_repaint();
                    }
                })?;
            self.pending = Some(receiver);
        }
        Ok(())
    }

    pub fn request(&mut self, context: &Context) -> Result<()> {
        *self
            .wake
            .lock()
            .expect("font wake-up only stores a cloned context") = Some(context.clone());
        self.start(None)?;
        // The worker may have finished before a context was registered.
        self.poll()
    }

    fn accept(&mut self, fonts: PreparedFonts) {
        self.ready = Some(fonts.native);
        self.initial = fonts.initial;
        self.pending = None;
    }

    pub fn poll(&mut self) -> Result<()> {
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(fonts) => self.accept(fonts),
                Err(mpsc::TryRecvError::Empty) => {}
                Err(error) => {
                    self.pending = None;
                    return Err(error.into());
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn names(&self) -> Option<&[String]> {
        self.ready.as_ref().map(|fonts| fonts.names.as_slice())
    }

    /// Installs UI weights and one editor family, preserving TTC indices and variations.
    pub fn definitions(&mut self, name: &str) -> Result<FontDefinitions> {
        self.definitions_for_stack(&[name.to_owned()])
    }

    /// Resolves configured families before automatic bundled fallbacks.
    /// Uninstalled families are skipped; an entirely unavailable stack is an error.
    pub fn definitions_for_stack(&mut self, names: &[String]) -> Result<FontDefinitions> {
        if self.ready.is_none() {
            match self.pending.take() {
                Some(receiver) => self.accept(receiver.recv()?),
                None => self.ready = Some(SystemFonts::new()),
            }
        }
        if let Some((prepared_name, definitions)) = self.initial.take()
            && prepared_name == names
        {
            return definitions;
        }
        let native = self
            .ready
            .as_mut()
            .expect("native catalog initialized above");
        font_definitions::definitions(native, names)
    }
}
