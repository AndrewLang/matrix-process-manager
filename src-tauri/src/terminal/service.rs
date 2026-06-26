use crate::models::CommandError;
use crate::terminal::manager::TerminalManager;
use crate::terminal::models::{
    TerminalResizeRequest, TerminalSessionInfo, TerminalSessionRequest, TerminalStartRequest,
    TerminalStartResponse, TerminalStopRequest, TerminalWriteRequest,
};
use crate::terminal::repository::TerminalRepository;
use crate::terminal::session::TerminalSession;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

pub struct TerminalService {
    manager: Mutex<TerminalManager>,
    repository: TerminalRepository,
}

impl TerminalService {
    pub fn new() -> Result<Self, CommandError> {
        Ok(Self {
            manager: Mutex::new(TerminalManager::new()),
            repository: TerminalRepository::new()?,
        })
    }

    pub fn start_session(
        &self,
        request: TerminalStartRequest,
        app_handle: AppHandle,
    ) -> Result<TerminalStartResponse, CommandError> {
        let session_id = self.next_session_id();
        let working_directory = self.working_directory(request.working_directory)?;
        let shell = request.shell;
        let session = TerminalSession::start(
            session_id.clone(),
            shell,
            working_directory.clone(),
            request.cols.unwrap_or(100).max(20),
            request.rows.unwrap_or(30).max(6),
            app_handle,
        )?;

        self.repository.record_started(
            session.session_id(),
            &format!("{:?}", session.shell()),
            session.working_directory(),
        )?;

        self.manager
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal sessions are unavailable"))?
            .create_session(session);

        Ok(TerminalStartResponse {
            session_id,
            shell,
            working_directory,
        })
    }

    pub fn write_input(&self, request: TerminalWriteRequest) -> Result<(), CommandError> {
        let manager = self
            .manager
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal sessions are unavailable"))?;
        let session = manager.get_session(&request.session_id)?;
        session.write_input(&request.input)
    }

    pub fn resize_session(&self, request: TerminalResizeRequest) -> Result<(), CommandError> {
        let manager = self
            .manager
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal sessions are unavailable"))?;
        let session = manager.get_session(&request.session_id)?;
        session.resize(request.cols.max(20), request.rows.max(6))
    }

    pub fn stop_session(&self, request: TerminalStopRequest) -> Result<(), CommandError> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal sessions are unavailable"))?;
        if let Some(mut session) = manager.remove_session(&request.session_id) {
            session.stop()?;
            self.repository.record_stopped(&request.session_id)?;
        }
        Ok(())
    }

    pub fn get_session(
        &self,
        request: TerminalSessionRequest,
    ) -> Result<TerminalSessionInfo, CommandError> {
        self.manager
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal sessions are unavailable"))?
            .session_info(&request.session_id)
    }

    pub fn active_session(&self) -> Result<Option<TerminalSessionInfo>, CommandError> {
        Ok(self
            .manager
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal sessions are unavailable"))?
            .active_session_info())
    }

    pub fn set_active_session(&self, request: TerminalSessionRequest) -> Result<(), CommandError> {
        self.manager
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal sessions are unavailable"))?
            .set_active_session(&request.session_id)
    }

    fn working_directory(&self, requested: Option<String>) -> Result<String, CommandError> {
        match requested.filter(|path| !path.trim().is_empty()) {
            Some(path) => Ok(path),
            None => std::env::current_dir()
                .map(|path| path.to_string_lossy().to_string())
                .map_err(|error| CommandError::terminal_failed(error.to_string())),
        }
    }

    fn next_session_id(&self) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("terminal-{nanos}")
    }
}
