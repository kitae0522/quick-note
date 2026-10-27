//! Search results are rebuilt on edits and query changes, never on idle editor frames.
use crate::i18n;
use crate::i18n::Messages;
use crate::model;
use crate::model::Note;
use crate::model::TextMatch;
use crate::shortcut::Command;

#[derive(Default)]
pub(super) struct DocumentFind {
    pub open: bool,
    pub show_replace: bool,
    pub query: String,
    pub replacement: String,
    pub match_case: bool,
    pub focus: bool,
    pub matches: Vec<TextMatch>,
    pub active: usize,
    pub reveal: bool,
    pub replaced: Option<usize>,
    pub pending: Option<Command>,
    revision: Option<(String, f64)>,
}

impl DocumentFind {
    pub fn status(&self, text: &Messages) -> String {
        if let Some(count) = self.replaced {
            i18n::format(&text.find_replaced, &[("count", &count.to_string())])
        } else if self.query.is_empty() {
            String::new()
        } else if self.matches.is_empty() {
            text.find_no_matches.clone()
        } else {
            i18n::format(
                &text.find_match_count,
                &[
                    ("current", &(self.active + 1).to_string()),
                    ("total", &self.matches.len().to_string()),
                ],
            )
        }
    }

    pub fn invalidate(&mut self) {
        self.revision = None;
        self.replaced = None;
    }

    pub fn refresh(&mut self, note: &Note) {
        if self
            .revision
            .as_ref()
            .is_some_and(|(id, revision)| *id == note.id && *revision == note.updated_at)
        {
            return;
        }
        self.matches = model::find_matches(&note.body, &self.query, self.match_case);
        let (caret, _) = note.selection.char_range(&note.body);
        self.active = self
            .matches
            .iter()
            .position(|found| found.characters.start >= caret)
            .unwrap_or(0);
        self.revision = Some((note.id.clone(), note.updated_at));
    }

    pub fn close(&mut self) {
        self.open = false;
        self.pending = None;
        self.matches.clear();
        self.invalidate();
    }
}
