//! Font filtering and ranking run only when the query or native catalog changes.
const PRESETS: [&str; 5] = ["SF Pro", "Pretendard", "Noto Sans CJK", "Menlo", "Consolas"];

#[derive(Default)]
pub(crate) struct FontChoices {
    query: Option<String>,
    native_ready: bool,
    names: Vec<String>,
}

impl FontChoices {
    pub fn update(&mut self, native: Option<&[String]>, query: &str) {
        let query = query.to_lowercase();
        if self.query.as_ref() == Some(&query) && self.native_ready == native.is_some() {
            return;
        }
        self.names = PRESETS
            .into_iter()
            .map(str::to_owned)
            .chain(native.unwrap_or_default().iter().cloned())
            .filter(|name| name.to_lowercase().contains(&query))
            .collect();
        self.names.sort_by_cached_key(|name| {
            let name = name.to_lowercase();
            let preset = if query.is_empty() {
                PRESETS
                    .iter()
                    .position(|preset| preset.eq_ignore_ascii_case(&name))
                    .unwrap_or(PRESETS.len())
            } else {
                0
            };
            (name != query, !name.starts_with(&query), preset, name)
        });
        self.names.dedup();
        self.query = Some(query);
        self.native_ready = native.is_some();
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }
}
