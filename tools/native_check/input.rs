//! Isolated native-window driver. Production never imports this fixture harness.
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::ImeEvent;
use eframe::egui::Modifiers;
use eframe::egui::PointerButton;
use eframe::egui::RawInput;
use eframe::egui::ViewportCommand;
use serde_json::Value;
use std::fs;
use std::time::Instant;

use super::probe::Probe;

impl Probe {
    pub(super) fn feed_input(&mut self, context: &Context, input: &mut RawInput) {
        // A foreground fixture window must never record the person's unrelated typing.
        let mut clipboard_received = false;
        #[cfg(target_os = "macos")]
        let mut native_received = false;
        #[cfg(not(target_os = "macos"))]
        let native_received = false;
        input.events.retain(|event| {
            #[cfg(target_os = "macos")]
            if self
                .native_expected
                .as_ref()
                .is_some_and(|expected| super::appkit::matches(expected, event))
            {
                native_received = true;
                self.queued_events.push(event.clone());
                return true;
            }
            let clipboard = self.clipboard_requested
                && matches!(event, Event::Cut | Event::Copy | Event::Paste(_));
            clipboard_received |= clipboard;
            if clipboard {
                self.queued_events.push(event.clone());
            }
            matches!(event, Event::WindowFocused(_))
        });
        if clipboard_received {
            self.clipboard_requested = false;
        }
        if self.native_pending.is_some() {
            if native_received
                || self
                    .native_pending
                    .is_some_and(|(_, start)| start.elapsed().as_secs_f32() > 1.0)
            {
                self.native_delivered = native_received;
                self.pending = self.native_pending.take();
                self.native_expected = None;
            }
            context.request_repaint_after(std::time::Duration::from_millis(1));
            return;
        }
        if !self.ready {
            return;
        }
        if !input.focused && !self.native_mode {
            self.focus_fixture(context);
        }
        // Replay queued input only during focused UI passes. Read each command once so an
        // atomic replacement cannot change its kind between the resident and editor handlers.
        if input.focused {
            input.events.extend(self.queued_events.iter().cloned());
        }
        let Ok(bytes) = fs::read(self.output.join("command.json")) else {
            return;
        };
        let Ok(command) = serde_json::from_slice::<Value>(&bytes) else {
            return;
        };
        #[cfg(target_os = "macos")]
        if self.feed_resident(context, &command) {
            return;
        }
        if !input.focused {
            return;
        }
        let Some(sequence) = command["sequence"]
            .as_u64()
            .filter(|sequence| *sequence > self.sequence)
        else {
            return;
        };
        self.sequence = sequence;
        self.pending = Some((sequence, Instant::now()));
        self.menu_key_handled = None;
        let injected_start = input.events.len();
        match command["type"].as_str().unwrap_or("") {
            #[cfg(target_os = "macos")]
            "native_pointer" | "native_key" => {
                self.native_mode = true;
                self.native_pending = self.pending.take();
                self.native_expected = Some(command.clone());
                self.native_delivered = false;
                if let Err(error) = super::appkit::post(&command) {
                    self.fail(error);
                }
                context.request_repaint();
            }
            #[cfg(target_os = "macos")]
            "menu" => {
                self.clipboard_requested = command["clipboard"].as_bool().unwrap_or(false);
                if self.clipboard_requested && self.clipboard_backup.is_none() {
                    match super::clipboard::ClipboardBackup::capture() {
                        Ok(backup) => self.clipboard_backup = Some(backup),
                        Err(error) => {
                            self.fail(error);
                            return;
                        }
                    }
                }
                if let Err(error) = super::menu::activate(
                    command["menu"].as_str().unwrap_or_default(),
                    command["item"].as_str().unwrap_or_default(),
                ) {
                    self.fail(error);
                }
            }
            #[cfg(target_os = "macos")]
            "restore_clipboard" => {
                if let Err(error) = self.restore_clipboard() {
                    self.fail(error);
                }
            }
            #[cfg(target_os = "macos")]
            "menu_key" => match super::menu::key_equivalent(
                command["key"].as_str().unwrap_or_default(),
                command["shift"].as_bool().unwrap_or(false),
            ) {
                Ok(handled) => self.menu_key_handled = Some(handled),
                Err(error) => self.fail(error),
            },
            "text" => {
                input
                    .events
                    .push(Event::Text(command["text"].as_str().unwrap_or("").into()));
            }
            "key" => match super::key_input::events(&command) {
                Ok(events) => input.events.extend(events),
                Err(error) => self.fail(error),
            },

            "pointer" => {
                let pos = eframe::egui::pos2(
                    command["x"].as_f64().unwrap_or(0.0) as f32,
                    command["y"].as_f64().unwrap_or(0.0) as f32,
                );
                input.events.push(Event::PointerMoved(pos));
                if let Some(pressed) = command["pressed"].as_bool() {
                    input.events.push(Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    });
                }
            }
            "preedit" | "commit" => {
                let text = command["text"].as_str().unwrap_or_default().to_owned();
                let event = if command["type"] == "preedit" {
                    ImeEvent::Preedit {
                        text,
                        active_range_chars: None,
                    }
                } else {
                    ImeEvent::Commit(text)
                };
                input.events.push(Event::Ime(event));
            }
            "theme" => {
                if let Err(error) = self
                    .app
                    .set_theme(command["id"].as_str().unwrap_or("helium"), context)
                {
                    self.fail(error);
                }
            }
            "screenshot" => {
                let name = command["name"].as_str().unwrap_or("capture");
                if name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                {
                    self.request_screenshot(context, name.to_owned());
                }
            }
            "resize" => context.send_viewport_cmd(ViewportCommand::InnerSize(eframe::egui::vec2(
                command["width"].as_f64().unwrap_or(920.0) as f32,
                command["height"].as_f64().unwrap_or(600.0) as f32,
            ))),
            "quit" => context.send_viewport_cmd(ViewportCommand::Close),
            "report" => {}
            other => self.fail(format!("Unknown command: {other}")),
        }
        self.queued_events
            .extend(input.events[injected_start..].iter().cloned());
    }
}
