mod lines;
mod note;
mod search_index;
mod selection;
mod text_search;
pub(crate) use search_index::SearchIndex;

pub use lines::Lines;
pub use note::Note;
pub use note::NoteSummary;
pub use note::note_title;
pub use selection::Selection;
pub use text_search::TextMatch;
pub use text_search::find_matches;

mod cursor_location;
pub use cursor_location::CursorLocation;
