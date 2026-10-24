//! Search-only normalized snapshots never replace the original persisted note text.
use super::Note;
use super::NoteSummary;

struct Entry {
    summary: NoteSummary,
    body: String,
    title: String,
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
                        title: summary.title.to_lowercase(),
                        summary,
                        body: note.body.to_lowercase(),
                    }
                })
                .collect(),
        }
    }

    pub fn search(&self, query: &str, untitled: &str) -> Vec<NoteSummary> {
        let untitled = untitled.to_lowercase();
        self.entries
            .iter()
            .filter(|entry| {
                let title = if entry.summary.is_blank {
                    &untitled
                } else {
                    &entry.title
                };
                entry.body.contains(query) || title.contains(query)
            })
            .map(|entry| entry.summary.clone())
            .collect()
    }
}
