use crate::models::CommandError;
use crate::terminal::models::TerminalSessionInfo;
use crate::terminal::session::TerminalSession;
use std::collections::HashMap;

pub struct TerminalManager {
    sessions: HashMap<String, TerminalSession>,
    active_session_id: Option<String>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
        }
    }

    pub fn create_session(&mut self, session: TerminalSession) -> String {
        let session_id = session.session_id().to_string();
        self.sessions.insert(session_id.clone(), session);
        self.active_session_id = Some(session_id.clone());
        session_id
    }

    pub fn remove_session(&mut self, session_id: &str) -> Option<TerminalSession> {
        let session = self.sessions.remove(session_id);
        if self.active_session_id.as_deref() == Some(session_id) {
            self.active_session_id = self.sessions.keys().next().cloned();
        }
        session
    }

    pub fn get_session(&self, session_id: &str) -> Result<&TerminalSession, CommandError> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| CommandError::terminal_failed("terminal session was not found"))
    }

    pub fn active_session(&self) -> Option<&TerminalSession> {
        self.active_session_id
            .as_deref()
            .and_then(|session_id| self.sessions.get(session_id))
    }

    pub fn set_active_session(&mut self, session_id: &str) -> Result<(), CommandError> {
        if self.sessions.contains_key(session_id) {
            self.active_session_id = Some(session_id.to_string());
            Ok(())
        } else {
            Err(CommandError::terminal_failed(
                "terminal session was not found",
            ))
        }
    }

    pub fn session_info(&self, session_id: &str) -> Result<TerminalSessionInfo, CommandError> {
        self.get_session(session_id)
            .map(|session| self.info_from_session(session))
    }

    pub fn active_session_info(&self) -> Option<TerminalSessionInfo> {
        self.active_session()
            .map(|session| self.info_from_session(session))
    }

    fn info_from_session(&self, session: &TerminalSession) -> TerminalSessionInfo {
        TerminalSessionInfo {
            session_id: session.session_id().to_string(),
            shell: session.shell(),
            working_directory: session.working_directory().to_string(),
            active: self.active_session_id.as_deref() == Some(session.session_id()),
        }
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}
