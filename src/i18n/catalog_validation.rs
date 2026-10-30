//! Nested catalogs retain the same nonempty-string and named-placeholder contracts.
use super::message;
use super::template;
use crate::storage::Result;
use serde_json::Value;

pub(super) fn validate(reference: &Value, candidate: &Value, path: &str) -> Result<()> {
    if let Some(fields) = reference.as_object() {
        for (key, expected) in fields {
            let path = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            validate(expected, &candidate[key], &path)?;
        }
        return Ok(());
    }
    let actual = candidate
        .as_str()
        .expect("typed catalog leaves are strings");
    if actual.trim().is_empty() {
        return Err(message(|m| &m.error_locale_empty).arg("key", path).into());
    }
    let expected = template::fields(
        reference
            .as_str()
            .expect("typed catalog leaves are strings"),
    )?;
    if template::fields(actual)? != expected {
        return Err(message(|m| &m.error_locale_placeholders)
            .arg("key", path)
            .arg(
                "expected",
                expected.into_iter().collect::<Vec<_>>().join(", "),
            )
            .into());
    }
    Ok(())
}

pub(super) fn merge(base: &mut Value, supplied: Value) {
    if let (Some(base), Some(supplied)) = (base.as_object_mut(), supplied.as_object()) {
        for (key, value) in supplied {
            merge(
                base.entry(key.clone()).or_insert(Value::Null),
                value.clone(),
            );
        }
    } else {
        *base = supplied;
    }
}
