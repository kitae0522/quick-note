/// Destination for keyboard input after a note becomes active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusTarget {
    Editor,
    Tab,
}
