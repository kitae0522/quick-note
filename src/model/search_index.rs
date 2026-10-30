//! Search-only normalized snapshots never replace the original persisted note text.
use super::Note;
use super::NoteSummary;
use super::SearchPreview;

struct Entry {
    summary: NoteSummary,
    body: Option<String>,
    title: String,
    original: String,
}

pub(crate) struct SearchIndex {
    entries: Vec<Entry>,
}

impl SearchIndex {
    pub fn new(notes: Vec<Note>) -> Self {
        Self {
            entries: notes
                .into_iter()
                .map(|note| {
                    let summary = note.summary();
                    Entry {
                        title: caseless::default_case_fold_str(&summary.title),
                        summary,
                        body: None,
                        original: note.body,
                    }
                })
                .collect(),
        }
    }

    pub fn search(&mut self, query: &str, untitled: &str) -> Vec<NoteSummary> {
        let untitled = caseless::default_case_fold_str(untitled);
        for entry in &mut self.entries {
            if entry.body.is_none() {
                entry.body = Some(caseless::default_case_fold_str(&entry.original));
            }
        }
        let mut notes: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| {
                let title = if entry.summary.is_blank {
                    &untitled
                } else {
                    &entry.title
                };
                entry.body.as_ref().is_some_and(|body| body.contains(query))
                    || title.contains(query)
            })
            .map(|entry| entry.summary.clone())
            .collect();
        notes.sort_by(|a, b| b.updated_at.total_cmp(&a.updated_at));
        notes
    }

    pub fn upsert(&mut self, note: &Note) {
        let entry = Entry {
            summary: note.summary(),
            title: caseless::default_case_fold_str(&note.title()),
            body: None,
            original: note.body.clone(),
        };
        if let Some(previous) = self
            .entries
            .iter_mut()
            .find(|entry| entry.summary.id == note.id)
        {
            *previous = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.entries.retain(|entry| entry.summary.id != id);
    }

    pub fn preview(&self, id: &str, query: &str) -> Option<SearchPreview> {
        self.entries
            .iter()
            .find(|entry| entry.summary.id == id)
            .map(|entry| SearchPreview::new(&entry.original, query))
    }
}
