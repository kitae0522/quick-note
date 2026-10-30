//! Typed copy for quick capture and macOS residency.
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureMessages {
    pub new_note: String,
    pub clipboard: String,
    pub clipboard_empty: String,
    pub clipboard_saved: String,
    pub error_clipboard: String,
    pub recent_notes: String,
    pub open_at_login: String,
    pub login_requires_approval: String,
    pub login_unavailable: String,
    pub login_settings: String,
    pub error_login: String,
}
