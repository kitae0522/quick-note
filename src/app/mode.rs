//! Source and rendered document modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Edit,
    Preview,
}
