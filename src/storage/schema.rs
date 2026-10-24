//! Open, validate and migrate the UTF-8 database before any note is shown.
use super::Result;
use crate::i18n;
use crate::model;
use rusqlite::Connection;
use rusqlite::params;
use std::fs;
use std::path::Path;
use std::time::Duration;

pub(super) fn open_connection(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut connection = Connection::open(path)?;
    connection.busy_timeout(Duration::from_millis(50))?;
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > 2 {
        return Err(i18n::message(|m| &m.error_schema_newer)
            .arg("version", version)
            .into());
    }
    connection.pragma_update(None, "encoding", "UTF-8")?;
    let encoding: String = connection.pragma_query_value(None, "encoding", |row| row.get(0))?;
    if encoding != "UTF-8" {
        return Err(i18n::message(|m| &m.error_database_encoding).into());
    }
    private_file(path)?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    #[cfg(target_os = "macos")]
    connection.pragma_update(None, "fullfsync", "ON")?;

    if version == 1 {
        let backup = path.with_extension("v1-backup.sqlite3");
        if !backup.exists() {
            let backup_path = backup
                .to_str()
                .ok_or(i18n::message(|m| &m.error_backup_path))?;
            connection.execute("VACUUM INTO ?1", [backup_path])?;
        }
        private_file(&backup)?;
    }
    if version < 2 {
        let transaction = connection.transaction()?;
        if version == 0 {
            transaction.execute_batch(
                "CREATE TABLE IF NOT EXISTS notes (
                    id TEXT PRIMARY KEY NOT NULL, body TEXT NOT NULL,
                    created_at REAL NOT NULL, updated_at REAL NOT NULL,
                    selection_location INTEGER NOT NULL DEFAULT 0,
                    selection_length INTEGER NOT NULL DEFAULT 0,
                    title TEXT NOT NULL DEFAULT 'Untitled');
                    CREATE TABLE IF NOT EXISTS app_state (
                    key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);",
            )?;
        } else {
            transaction.execute(
                "ALTER TABLE notes ADD COLUMN title TEXT NOT NULL DEFAULT 'Untitled'",
                [],
            )?;
            let titles: Vec<(String, String)> = transaction
                .prepare("SELECT id, body FROM notes")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?;
            for (id, body) in titles {
                transaction.execute(
                    "UPDATE notes SET title = ?1 WHERE id = ?2",
                    params![model::note_title(&body), id],
                )?;
            }
        }
        transaction.pragma_update(None, "user_version", 2)?;
        transaction.commit()?;
    }
    Ok(connection)
}

fn private_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
