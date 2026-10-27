//! Find actions change selections; replacements commit once through the normal recovery path.
use super::Mode;
use super::QuickNoteApp;
use crate::i18n;
use crate::model::Selection;
use crate::shortcut::Command;
use crate::ui::editor_mutation;
use eframe::egui::Context;
use eframe::egui::Event;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::Modifiers;
use eframe::egui::TextBuffer;

impl QuickNoteApp {
    pub(super) fn find_command(&mut self, command: Command, context: &Context) {
        if self.composing
            || context.input(|input| {
                input
                    .events
                    .iter()
                    .any(|event| matches!(event, Event::Ime(_)))
            })
        {
            self.document_find.pending = Some(command);
            context.request_repaint();
            return;
        }
        match command {
            Command::Find | Command::Replace => {
                let (start, end) = self.note.selection.char_range(&self.note.body);
                let selected = self.note.body.char_range(start.into()..end.into());
                if !selected.is_empty() && !selected.contains(['\r', '\n']) {
                    self.document_find.query = selected.to_owned();
                }
                self.document_find.open = true;
                self.document_find.show_replace = command == Command::Replace;
                self.document_find.focus = true;
                self.document_find.invalidate();
                self.document_find.refresh(&self.note);
                self.select_find_match();
                self.mode = Mode::Edit;
                self.search_open = false;
                self.settings = None;
                self.settings_request = None;
                self.focus_editor = false;
            }
            Command::FindNext | Command::FindPrevious => {
                if self.document_find.query.is_empty() {
                    self.find_command(Command::Find, context);
                    return;
                }
                if !self.document_find.open {
                    self.document_find.open = true;
                    self.document_find.focus = false;
                    self.document_find.invalidate();
                    self.mode = Mode::Edit;
                    self.search_open = false;
                    self.settings = None;
                    self.settings_request = None;
                    self.focus_editor = true;
                }
                self.document_find.refresh(&self.note);
                let count = self.document_find.matches.len();
                if count > 0 {
                    let (start, end) = self.note.selection.char_range(&self.note.body);
                    let matches = &self.document_find.matches;
                    self.document_find.active = if command == Command::FindPrevious {
                        matches
                            .partition_point(|found| found.characters.start < start)
                            .checked_sub(1)
                            .unwrap_or(count - 1)
                    } else {
                        matches.partition_point(|found| found.characters.start < end) % count
                    };
                    self.document_find.replaced = None;
                    self.select_find_match();
                }
            }
            _ => {}
        }
    }

    pub(super) fn finish_find_navigation(&mut self, context: &Context) {
        if !self.composing && self.document_find.pending.is_some() {
            // The next frame sees the complete document, including this frame's IME commit.
            context.request_repaint();
        }
    }

    pub(super) fn find_keys(&mut self, context: &Context) {
        if let Some(command) = self.document_find.pending.take() {
            self.find_command(command, context);
        }
        if !self.document_find.open {
            return;
        }
        let query = context.memory(|memory| memory.has_focus(Id::unique("document-find-query")));
        let replacement =
            context.memory(|memory| memory.has_focus(Id::unique("document-find-replacement")));
        if query && context.input_mut(|input| input.consume_key(Modifiers::SHIFT, Key::Enter)) {
            self.find_command(Command::FindPrevious, context);
        } else if (query || replacement)
            && context.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter))
        {
            if replacement {
                self.replace_find(false, context);
            } else {
                self.find_command(Command::FindNext, context);
            }
        }
    }

    pub(super) fn select_find_match(&mut self) {
        if let Some(found) = self.document_find.matches.get(self.document_find.active) {
            let selection = Selection::from_char_range(
                &self.note.body,
                found.characters.start,
                found.characters.end,
            );
            if self.note.selection != selection {
                self.note.selection = selection;
                if !self.dirty
                    && let Err(error) = self.store.save_selection(&self.note)
                {
                    self.error = Some(
                        i18n::message(|m| &m.error_cursor_save)
                            .cause("error", error)
                            .into(),
                    );
                }
            }
            self.restore_cursor = true;
            self.document_find.reveal = true;
        }
    }

    pub(super) fn replace_find(&mut self, all: bool, context: &Context) {
        self.document_find.refresh(&self.note);
        let Some(found) =
            self.document_find
                .matches
                .get(if all { 0 } else { self.document_find.active })
        else {
            return;
        };
        let mut body;
        let caret = found.characters.start + self.document_find.replacement.chars().count();
        let count = if all {
            body = String::with_capacity(self.note.body.len());
            let mut end = 0;
            for found in &self.document_find.matches {
                body.push_str(&self.note.body[end..found.bytes.start]);
                body.push_str(&self.document_find.replacement);
                end = found.bytes.end;
            }
            body.push_str(&self.note.body[end..]);
            self.document_find.matches.len()
        } else {
            body = self.note.body.clone();
            body.replace_range(found.bytes.clone(), &self.document_find.replacement);
            1
        };
        let changed = editor_mutation::replace(context, &mut self.note, body, caret);
        if changed {
            self.dirty = true;
            self.markdown_cache = None;
            let _saved = self.persist();
            self.restore_cursor = true;
            self.document_find.invalidate();
            self.document_find.refresh(&self.note);
            self.editor_metrics.saved(&self.note);
        }
        if all {
            self.document_find.replaced = Some(count);
        } else if !changed {
            self.find_command(Command::FindNext, context);
        } else {
            self.select_find_match();
        }
        context.request_repaint();
    }

    pub(super) fn close_find(&mut self) {
        self.document_find.close();
        self.focus_editor = true;
        self.restore_cursor = true;
    }
}
