//! Isolated text/cursor fixtures shared by the paired-input behavior checks.
use quick_note::app::QuickNoteApp;
use quick_note::model::Selection;
use quick_note::storage::Result;

pub(super) fn editor(
    body: &str,
    start: usize,
    end: usize,
) -> Result<(tempfile::TempDir, QuickNoteApp)> {
    let directory = tempfile::tempdir()?;
    let mut app = QuickNoteApp::try_new(directory.path().to_owned())?;
    app.note.body = body.into();
    app.note.selection = Selection::from_char_range(body, start, end);
    assert!(app.persist());
    Ok((directory, app))
}
