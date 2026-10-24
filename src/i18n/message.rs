//! Keep messages and their causes structured until the current language renders them.
use super::Messages;
use super::english;
use super::template;
use std::error::Error as StdError;
use std::fmt;

pub type Error = Box<dyn StdError + Send + Sync>;

#[derive(Debug)]
enum Argument {
    Text(String),
    Cause(Error),
}

#[derive(Debug)]
pub struct Message {
    selector: fn(&Messages) -> &str,
    arguments: Vec<(&'static str, Argument)>,
}

#[must_use]
pub fn message(selector: fn(&Messages) -> &str) -> Message {
    Message {
        selector,
        arguments: Vec::new(),
    }
}

impl Message {
    #[must_use]
    pub fn arg(mut self, name: &'static str, value: impl ToString) -> Self {
        self.arguments
            .push((name, Argument::Text(value.to_string())));
        self
    }

    #[must_use]
    pub fn cause(mut self, name: &'static str, error: impl Into<Error>) -> Self {
        self.arguments.push((name, Argument::Cause(error.into())));
        self
    }
}

pub(super) fn render(error: &(dyn StdError + 'static), text: &Messages) -> String {
    let Some(message) = error.downcast_ref::<Message>() else {
        return error.to_string();
    };
    let values: Vec<_> = message
        .arguments
        .iter()
        .map(|(name, value)| {
            (
                *name,
                match value {
                    Argument::Text(value) => value.clone(),
                    Argument::Cause(error) => render(error.as_ref(), text),
                },
            )
        })
        .collect();
    let values: Vec<_> = values
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();
    template::format((message.selector)(text), &values)
}

impl fmt::Display for Message {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&render(self, english()))
    }
}

impl StdError for Message {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.arguments.iter().find_map(|(_, value)| match value {
            Argument::Cause(error) => Some(error.as_ref() as &(dyn StdError + 'static)),
            Argument::Text(_) => None,
        })
    }
}
