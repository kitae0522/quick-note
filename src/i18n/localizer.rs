//! UTF-8 catalogs have typed fields and identical placeholder contracts.
use super::Language;
use super::Messages;
use super::message;
use super::template;
use crate::storage::Result;
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

const ENGLISH: &str = include_str!("../../i18n/en.json");
const KOREAN: &str = include_str!("../../i18n/ko.json");

#[must_use]
pub fn english() -> &'static Messages {
    static TEXT: OnceLock<Messages> = OnceLock::new();
    TEXT.get_or_init(|| serde_json::from_str(ENGLISH).expect("bundled English catalog is tested"))
}

#[derive(Clone)]
pub struct Localizer {
    pub language: Language,
    text: Messages,
}

impl Default for Localizer {
    fn default() -> Self {
        Self {
            language: Language::English,
            text: english().clone(),
        }
    }
}

impl Localizer {
    pub fn bundled(language: Language) -> Result<Self> {
        Self::decode(
            language,
            if language == Language::Korean {
                KOREAN
            } else {
                ENGLISH
            }
            .as_bytes(),
        )
    }

    pub fn decode(language: Language, bytes: &[u8]) -> Result<Self> {
        let text: Messages = serde_json::from_slice(bytes)?;
        let reference = serde_json::to_value(english())?;
        let candidate = serde_json::to_value(&text)?;
        for (key, expected) in reference
            .as_object()
            .expect("Messages serialize to an object")
        {
            if candidate[key]
                .as_str()
                .expect("message is a string")
                .trim()
                .is_empty()
            {
                return Err(message(|m| &m.error_locale_empty).arg("key", key).into());
            }
            let expected = template::fields(expected.as_str().expect("message is a string"))?;
            let actual = template::fields(candidate[key].as_str().expect("message is a string"))?;
            if actual != expected {
                return Err(message(|m| &m.error_locale_placeholders)
                    .arg("key", key)
                    .arg(
                        "expected",
                        expected.into_iter().collect::<Vec<_>>().join(", "),
                    )
                    .into());
            }
        }
        Ok(Self { language, text })
    }

    /// Optional data-folder catalogs override packaged files; absent files use bundled defaults.
    pub fn load(language: Language, data_directory: &Path) -> Result<Self> {
        let name = format!("{}.json", language.code());
        let mut paths = vec![data_directory.join("i18n").join(&name)];
        if let Some(parent) = std::env::current_exe()?.parent() {
            paths.push(parent.join("i18n").join(&name));
            paths.push(parent.join("../Resources/i18n").join(&name));
        }
        for path in paths {
            let loaded = match fs::read(&path) {
                Ok(bytes) => Self::decode_override(language, &bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => Err(error.into()),
            };
            return loaded.map_err(|error| {
                message(|m| &m.error_locale_file)
                    .arg("path", path.display())
                    .cause("error", error)
                    .into()
            });
        }
        Self::bundled(language)
    }

    fn decode_override(language: Language, bytes: &[u8]) -> Result<Self> {
        let supplied: BTreeMap<String, Value> = serde_json::from_slice(bytes)?;
        let mut base = serde_json::to_value(Self::bundled(language)?.text())?;
        base.as_object_mut()
            .expect("Messages serialize to an object")
            .extend(supplied);
        Self::decode(language, &serde_json::to_vec(&base)?)
    }

    #[must_use]
    pub fn text(&self) -> &Messages {
        &self.text
    }

    #[must_use]
    pub fn error(&self, error: &(dyn Error + 'static)) -> String {
        message::render(error, &self.text)
    }
}
