//! Per-context labels let applications localize generated code-block controls.
#[derive(Clone)]
pub struct CodeBlockLabels {
    pub read_only: String,
    pub copy: String,
    pub copied: String,
    pub copy_accessible: String,
    pub copied_accessible: String,
}

impl Default for CodeBlockLabels {
    fn default() -> Self {
        Self {
            read_only: "Code block (read only)".into(),
            copy: "Copy".into(),
            copied: "Copied".into(),
            copy_accessible: "Copy code".into(),
            copied_accessible: "Copied code. Copy again".into(),
        }
    }
}

impl CodeBlockLabels {
    pub fn install(self, context: &egui::Context) {
        context.data_mut(|data| data.insert_temp(egui::Id::unique("commonmark-code-labels"), self));
    }

    pub(crate) fn get(context: &egui::Context) -> Self {
        context.data(|data| data.get_temp(egui::Id::unique("commonmark-code-labels"))).unwrap_or_default()
    }
}
