//! UTF-8 exports, atomic file replacement and application data locations.
use crate::appearance::Theme;
use crate::i18n;
use crate::storage::Result;
use std::path::Path;
use std::path::PathBuf;

pub fn data_directory() -> Result<PathBuf> {
    let base = directories::BaseDirs::new().ok_or(i18n::message(|m| &m.error_data_directory))?;
    Ok(base.data_local_dir().join("Quick Note"))
}

pub(super) fn read_theme(directory: &Path, id: &str) -> Result<Theme> {
    if matches!(id, "helium" | "graphite") {
        return Ok(Theme::builtin(id));
    }
    if id.contains(['/', '\\']) || id.contains("..") || !id.ends_with(".json") {
        return Err(i18n::message(|m| &m.error_theme_path).into());
    }
    Theme::read(&directory.join("Themes").join(id))
}

pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or(i18n::message(|m| &m.error_destination))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(contents)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}

pub(super) fn safe_filename(title: &str, fallback: &str) -> String {
    let title: String = title
        .chars()
        .take(60)
        .map(|character| {
            if character.is_control() || "<>:\"/\\|?*".contains(character) {
                '-'
            } else {
                character
            }
        })
        .collect();
    let title = title.trim_matches([' ', '.']);
    if title.is_empty() {
        fallback.into()
    } else {
        title.into()
    }
}
