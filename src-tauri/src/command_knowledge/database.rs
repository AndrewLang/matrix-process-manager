use crate::models::CommandError;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

pub struct CommandKnowledgeDatabase;

impl CommandKnowledgeDatabase {
    pub fn open() -> Result<Connection, CommandError> {
        let path = Self::database_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        }
        Connection::open(path).map_err(|error| CommandError::terminal_failed(error.to_string()))
    }

    fn database_path() -> Result<PathBuf, CommandError> {
        let base = if cfg!(windows) {
            std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
        } else if cfg!(target_os = "macos") {
            std::env::var_os("HOME").map(|home| {
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
            })
        } else {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME")
                        .map(|home| PathBuf::from(home).join(".local").join("share"))
                })
        };

        base.map(|path| {
            path.join("Matrix Process Manager")
                .join("command-knowledge.sqlite")
        })
        .ok_or_else(|| CommandError::terminal_failed("application data directory is unavailable"))
    }
}
