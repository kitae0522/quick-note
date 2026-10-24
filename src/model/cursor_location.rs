/// One-based logical line and grapheme column in a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorLocation {
    pub line: usize,
    pub column: usize,
}
