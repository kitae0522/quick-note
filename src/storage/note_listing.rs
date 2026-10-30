//! Lists metadata without loading every note body for tab and library navigation.
use super::Result;
use super::Store;
use super::tab_order;
use crate::model::NoteSummary;

impl Store {
    pub fn summaries(&self) -> Result<Vec<NoteSummary>> {
        let mut notes =
            self.list_notes("deleted_at IS NULL AND closed = 0", "created_at, rowid")?;
        let positions = tab_order::positions(&self.connection)?;
        if !positions.is_empty() {
            notes.sort_by_cached_key(|note| positions.get(&note.id).copied().unwrap_or(usize::MAX));
        }
        Ok(notes)
    }

    pub fn all_summaries(&self) -> Result<Vec<NoteSummary>> {
        self.list_notes("deleted_at IS NULL", "updated_at DESC, rowid DESC")
    }

    pub fn recent_summaries(&self) -> Result<Vec<NoteSummary>> {
        self.list_notes("deleted_at IS NULL", "updated_at DESC, rowid DESC LIMIT 5")
    }

    pub fn deleted_summaries(&self) -> Result<Vec<NoteSummary>> {
        self.list_notes("deleted_at IS NOT NULL", "deleted_at DESC, rowid DESC")
    }

    fn list_notes(&self, filter: &str, order: &str) -> Result<Vec<NoteSummary>> {
        let mut statement = self.connection.prepare_cached(&format!(
            "SELECT id, title, updated_at, CASE WHEN title = 'Untitled' THEN body END \
            FROM notes WHERE {filter} ORDER BY {order}"
        ))?;
        Ok(statement
            .query_map([], |row| {
                Ok(NoteSummary {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    updated_at: row.get(2)?,
                    is_blank: row
                        .get::<_, Option<String>>(3)?
                        .is_some_and(|body| body.trim().is_empty()),
                })
            })?
            .collect::<rusqlite::Result<_>>()?)
    }
}
