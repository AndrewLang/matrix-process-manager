use crate::models::CommandError;
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct TerminalRepository {
    connection: Mutex<Connection>,
}

impl TerminalRepository {
    pub fn new() -> Result<Self, CommandError> {
        let repository = Self {
            connection: Mutex::new(
                Connection::open_in_memory()
                    .map_err(|error| CommandError::terminal_failed(error.to_string()))?,
            ),
        };
        repository.initialize()?;
        Ok(repository)
    }

    pub fn record_started(
        &self,
        session_id: &str,
        shell: &str,
        working_directory: &str,
    ) -> Result<(), CommandError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal repository is unavailable"))?;
        connection
            .execute(
                "insert into terminal_sessions (session_id, shell, working_directory, started_at) values (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                params![session_id, shell, working_directory],
            )
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        Ok(())
    }

    pub fn record_stopped(&self, session_id: &str) -> Result<(), CommandError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal repository is unavailable"))?;
        connection
            .execute(
                "update terminal_sessions set stopped_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') where session_id = ?1",
                params![session_id],
            )
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        Ok(())
    }

    fn initialize(&self) -> Result<(), CommandError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal repository is unavailable"))?;
        connection
            .execute(
                "create table if not exists terminal_sessions (session_id text primary key, shell text not null, working_directory text not null, started_at text not null, stopped_at text)",
                [],
            )
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        Ok(())
    }
}
