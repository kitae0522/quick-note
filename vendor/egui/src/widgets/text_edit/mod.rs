mod builder;
mod completion;
mod input_handler;
mod output;
mod state;
mod text_buffer;

pub use {
    crate::text_selection::TextCursorState,
    builder::TextEdit,
    completion::{CompletionOutput, CompletionPopup, CompletionQuery, Suggestion},
    input_handler::{TextEditInputHandler, TextEditMutation},
    output::TextEditOutput,
    state::TextEditState,
    text_buffer::TextBuffer,
};
