#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Korean,
}

impl Language {
    #[must_use]
    pub fn from_code(code: &str) -> Self {
        if code == "ko" {
            Self::Korean
        } else {
            Self::English
        }
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Korean => "ko",
        }
    }
}
