//! Logical-line indices survive redraws and cursor moves; text edits rebuild them once.
use crate::model::Lines;
use crate::model::Note;

pub struct EditorMetrics {
    note_id: String,
    revision: f64,
    bytes: usize,
    lines: Lines,
}

impl EditorMetrics {
    #[must_use]
    pub fn new(note: &Note) -> Self {
        Self {
            note_id: note.id.clone(),
            revision: note.updated_at,
            bytes: note.body.len(),
            lines: Lines::new(&note.body),
        }
    }

    pub(super) fn update(&mut self, note: &Note, changed: bool) {
        if changed
            || self.note_id != note.id
            || self.revision != note.updated_at
            || self.bytes != note.body.len()
        {
            *self = Self::new(note);
        }
    }

    /// A successful save changes the revision, not the already indexed editor contents.
    pub(crate) fn saved(&mut self, note: &Note) {
        if self.note_id == note.id && self.bytes == note.body.len() {
            self.revision = note.updated_at;
        }
    }

    #[must_use]
    pub fn lines(&self) -> &Lines {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Selection;

    #[test]
    fn edits_switches_and_external_revisions_rebuild_the_line_index() {
        let mut note = Note {
            id: "one".into(),
            body: "a\nb".into(),
            created_at: 0.0,
            updated_at: 1.0,
            selection: Selection::default(),
        };
        let mut metrics = EditorMetrics::new(&note);
        let starts = metrics.lines().starts.as_ptr();
        note.selection.location = 2;
        metrics.update(&note, false);
        assert_eq!(
            metrics.lines().starts.as_ptr(),
            starts,
            "cursor movement reuses the index"
        );
        note.body = "abc".into();
        metrics.update(&note, true);
        assert_eq!(
            metrics.lines().starts.len(),
            1,
            "same-byte-length edits invalidate"
        );
        note.updated_at = 2.0;
        metrics.saved(&note);
        let starts = metrics.lines().starts.as_ptr();
        metrics.update(&note, false);
        assert_eq!(metrics.lines().starts.as_ptr(), starts);
        note.body = "a\nb".into();
        note.updated_at = 3.0;
        metrics.update(&note, false);
        assert_eq!(metrics.lines().starts.len(), 2);
        note.id = "two".into();
        note.body = "a\rb".into();
        metrics.update(&note, false);
        assert_eq!(metrics.lines().starts, [0, 2]);
    }
}
