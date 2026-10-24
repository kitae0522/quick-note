//! Bounded UTF-8 configuration, default creation and all-or-nothing binding validation.
use super::ACTIONS;
use super::Platform;
use super::Shortcuts;
use super::bindings::Binding;
use super::key;
use crate::i18n;
use crate::storage::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

const DEFAULTS: &str = include_str!("../../shortcut/bindings.json");
const MAX_BYTES: u64 = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    bindings: BTreeMap<String, Vec<String>>,
}

impl Shortcuts {
    /// Creates the editable file only when absent. Existing files are never overwritten.
    pub fn load(directory: &Path, platform: Platform) -> Result<Self> {
        let folder = directory.join("shortcut");
        fs::create_dir_all(&folder)?;
        let path = folder.join("bindings.json");
        match File::options().write(true).create_new(true).open(&path) {
            Ok(mut file) => file.write_all(DEFAULTS.as_bytes())?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
        let mut bytes = Vec::new();
        File::open(path)?
            .take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)?;
        Self::from_bytes(&bytes, platform)
    }

    #[must_use]
    pub fn defaults(platform: Platform) -> Self {
        Self::from_bytes(DEFAULTS.as_bytes(), platform)
            .expect("bundled shortcut defaults are validated for both platforms")
    }

    pub fn from_bytes(bytes: &[u8], platform: Platform) -> Result<Self> {
        if bytes.len() as u64 > MAX_BYTES {
            return Err(i18n::message(|m| &m.error_shortcut_size).into());
        }
        let supplied: Configuration = serde_json::from_slice(bytes)?;
        if supplied.schema_version != 1 {
            return Err(i18n::message(|m| &m.error_shortcut_version).into());
        }
        let mut defaults: Configuration =
            serde_json::from_str(DEFAULTS).expect("bundled shortcut JSON is validated in tests");
        for (id, keys) in supplied.bindings {
            if !ACTIONS.iter().any(|action| action.id == id) {
                return Err(i18n::message(|m| &m.error_shortcut_action)
                    .arg("action", id)
                    .into());
            }
            defaults.bindings.insert(id, keys);
        }
        let mut bindings = Vec::new();
        let mut assigned = HashMap::new();
        for (index, action) in ACTIONS.iter().enumerate() {
            let sources = defaults
                .bindings
                .get(action.id)
                .expect("every catalog action has defaults");
            if sources.len() > 8 {
                return Err(i18n::message(|m| &m.error_shortcut_limit).into());
            }
            for source in sources {
                let keys = key::parse(source, platform)?;
                let fingerprint = key::fingerprint(keys.logical_key, keys.modifiers, platform);
                if let Some(previous) = assigned.insert(fingerprint, action.id) {
                    return Err(i18n::message(|m| &m.error_shortcut_conflict)
                        .arg("shortcut", source)
                        .arg("first", previous)
                        .arg("second", action.id)
                        .into());
                }
                bindings.push(Binding {
                    keys,
                    action: index,
                });
            }
        }
        Ok(Self { bindings, platform })
    }
}
