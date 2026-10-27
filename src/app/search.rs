//! Full-text search state and cached UTF-8 note queries.
use super::QuickNoteApp;
use crate::model::NoteSummary;
use crate::model::SearchIndex;
use crate::storage::Result;
use eframe::egui::Context;

impl QuickNoteApp {
    pub fn request_search(&mut self, context: &Context) {
        self.document_find.close();
        if !self.search_open && !self.settings_open() {
            self.previous_focus = context.memory(|memory| memory.focused());
        }
        self.focus_editor = false;
        self.settings = None;
        self.settings_request = None;
        self.search_open = true;
        self.search_index = 0;
        self.search_reveal = true;
        self.search_query = None;
        self.header.focus_search = context.content_rect().width() >= 620.0;
        self.search_focus_popup = !self.header.focus_search;
    }

    pub fn refresh_search(&mut self) -> Result<()> {
        let query = caseless::default_case_fold_str(self.header.query.trim());
        if self.search_query.as_deref() == Some(query.as_str()) {
            return Ok(());
        }
        self.search_results = if query.is_empty() {
            let mut notes = self.tabs.clone();
            notes.sort_by(|a, b| b.updated_at.total_cmp(&a.updated_at));
            notes
        } else {
            if self.search_cache.is_none() {
                self.search_cache = Some(SearchIndex::new(self.store.notes()?));
            }
            self.search_cache
                .as_ref()
                .expect("search index was initialized above")
                .search(&query, &self.locale.text().note_untitled)
        };
        self.search_index = self
            .search_index
            .min(self.search_results.len().saturating_sub(1));
        self.search_reveal = true;
        self.search_query = Some(query);
        Ok(())
    }

    #[must_use]
    pub fn search_results(&self) -> &[NoteSummary] {
        &self.search_results
    }
}
