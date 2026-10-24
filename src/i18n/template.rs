//! Expand named placeholders once; user text can never become another placeholder.
use super::message;
use crate::storage::Result;
use std::collections::BTreeSet;

enum Token<'a> {
    Literal(&'a str),
    Field(&'a str),
}

fn parse(mut text: &str) -> Result<Vec<Token<'_>>> {
    let original = text;
    let mut tokens = Vec::new();
    while let Some(index) = text.find(['{', '}']) {
        tokens.push(Token::Literal(&text[..index]));
        text = &text[index..];
        if text.starts_with("{{") || text.starts_with("}}") {
            tokens.push(Token::Literal(&text[..1]));
            text = &text[2..];
        } else if text.starts_with('{')
            && let Some(end) = text.find('}')
        {
            let field = &text[1..end];
            if field.is_empty()
                || !field
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_')
            {
                return Err(message(|m| &m.error_locale_template)
                    .arg("value", original)
                    .into());
            }
            tokens.push(Token::Field(field));
            text = &text[end + 1..];
        } else {
            return Err(message(|m| &m.error_locale_template)
                .arg("value", original)
                .into());
        }
    }
    tokens.push(Token::Literal(text));
    Ok(tokens)
}

pub(super) fn fields(text: &str) -> Result<BTreeSet<String>> {
    Ok(parse(text)?
        .into_iter()
        .filter_map(|token| match token {
            Token::Field(name) => Some(name.to_owned()),
            Token::Literal(_) => None,
        })
        .collect())
}

/// Formats a validated catalog template with named arguments.
///
/// # Panics
/// Panics if the application omits an argument or supplies an unvalidated template.
#[must_use]
pub fn format(template: &str, arguments: &[(&str, &str)]) -> String {
    let mut output = String::new();
    for token in parse(template).expect("catalog templates are validated before use") {
        match token {
            Token::Literal(value) => output.push_str(value),
            Token::Field(name) => output.push_str(
                arguments
                    .iter()
                    .find(|(key, _)| *key == name)
                    .expect("application supplies every named catalog argument")
                    .1,
            ),
        }
    }
    output
}
