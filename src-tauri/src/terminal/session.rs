use crate::models::CommandError;
use crate::terminal::indexer::TerminalIndexer;
use crate::terminal::models::{TerminalOutputEvent, TerminalOutputStream, TerminalShell};
use crate::terminal::parser::TerminalParser;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};

pub struct TerminalSession {
    session_id: String,
    shell: TerminalShell,
    working_directory: String,
    master: Box<dyn MasterPty + Send>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl TerminalSession {
    pub fn start(
        session_id: String,
        shell: TerminalShell,
        working_directory: String,
        cols: u16,
        rows: u16,
        app_handle: AppHandle,
    ) -> Result<Self, CommandError> {
        if !shell.allowed_on_current_platform() {
            return Err(CommandError::terminal_failed(
                "shell is not supported on this platform",
            ));
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;

        let mut command = CommandBuilder::new(shell.program());
        command.cwd(PathBuf::from(&working_directory));

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        let writer =
            Arc::new(Mutex::new(pair.master.take_writer().map_err(|error| {
                CommandError::terminal_failed(error.to_string())
            })?));

        Self::spawn_reader(session_id.clone(), reader, app_handle);

        Ok(Self {
            session_id,
            shell,
            working_directory,
            master: pair.master,
            writer,
            child,
        })
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn shell(&self) -> TerminalShell {
        self.shell
    }

    pub fn working_directory(&self) -> &str {
        &self.working_directory
    }

    pub fn write_input(&self, input: &str) -> Result<(), CommandError> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| CommandError::terminal_failed("terminal writer is unavailable"))?;
        writer
            .write_all(input.as_bytes())
            .map_err(|error| CommandError::terminal_failed(error.to_string()))?;
        writer
            .flush()
            .map_err(|error| CommandError::terminal_failed(error.to_string()))
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), CommandError> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| CommandError::terminal_failed(error.to_string()))
    }

    pub fn stop(&mut self) -> Result<(), CommandError> {
        self.child
            .kill()
            .map_err(|error| CommandError::terminal_failed(error.to_string()))
    }

    fn spawn_reader(session_id: String, mut reader: Box<dyn Read + Send>, app_handle: AppHandle) {
        thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        let _ = app_handle.emit(
                            "terminal://output",
                            TerminalOutputEvent {
                                session_id: session_id.clone(),
                                stream: TerminalOutputStream::Exit,
                                data: String::new(),
                            },
                        );
                        break;
                    }
                    Ok(count) => {
                        let data = TerminalIndexer::normalize_output(TerminalParser::decode(
                            &buffer[..count],
                        ));
                        let _ = app_handle.emit(
                            "terminal://output",
                            TerminalOutputEvent {
                                session_id: session_id.clone(),
                                stream: TerminalOutputStream::Stdout,
                                data,
                            },
                        );
                    }
                    Err(error) => {
                        let _ = app_handle.emit(
                            "terminal://output",
                            TerminalOutputEvent {
                                session_id: session_id.clone(),
                                stream: TerminalOutputStream::Stderr,
                                data: error.to_string(),
                            },
                        );
                        break;
                    }
                }
            }
        });
    }
}
