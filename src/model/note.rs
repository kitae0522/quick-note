use super::Selection;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub body: String,
    pub created_at: f64,
    pub updated_at: f64,
    pub selection: Selection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub updated_at: f64,
    pub is_blank: bool,
}

#[must_use]
pub fn note_title(body: &str) -> String {
    let line = body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");
    let hashes = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    let line = if (1..=6).contains(&hashes) && line[hashes..].starts_with(char::is_whitespace) {
        line[hashes..].trim()
    } else {
        line
    };
    if line.is_empty() {
        "Untitled".into()
    } else {
        line.into()
    }
}

impl Note {
    #[must_use]
    pub fn title(&self) -> String {
        note_title(&self.body)
    }
    #[must_use]
    pub fn summary(&self) -> NoteSummary {
        NoteSummary {
            id: self.id.clone(),
            title: self.title(),
            updated_at: self.updated_at,
            is_blank: self.body.trim().is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_preserves_content_and_handles_markdown_and_unicode() {
        assert_eq!(
            note_title("\n  # Café 아이디어 💡\nbody"),
            "Café 아이디어 💡"
        );
        assert_eq!(note_title("C#"), "C#");
        assert_eq!(note_title("##tag"), "##tag");
        assert_eq!(note_title("  \n"), "Untitled");
    }
}
