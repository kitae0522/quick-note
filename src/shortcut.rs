mod bindings;
mod catalog;
mod command;
mod configuration;
mod key;

pub use bindings::Shortcuts;
pub(crate) use catalog::ACTIONS;
pub use command::Command;
pub use key::Platform;
