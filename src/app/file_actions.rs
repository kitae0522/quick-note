//! Native export dialogs and user-facing export failures.
use super::QuickNoteApp;
use super::files;
use crate::i18n;

impl QuickNoteApp {
    pub(super) fn export_note(&mut self) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .set_title(&self.locale.text().export_note)
            .add_filter(self.locale.text().text_filter.as_str(), &["txt", "md"])
            .set_file_name(format!(
                "{}.md",
                files::safe_filename(
                    crate::ui::note_title(&self.note.summary(), self.locale.text()),
                    &self.locale.text().note_untitled
                )
            ))
            .save_file()
        else {
            return false;
        };
        match files::atomic_write(&path, self.note.body.as_bytes()) {
            Ok(()) => true,
            Err(error) => {
                self.error = Some(
                    i18n::message(|m| &m.error_export)
                        .cause("error", error)
                        .into(),
                );
                false
            }
        }
    }
}
