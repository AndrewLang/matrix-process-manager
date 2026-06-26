use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalStartRequest {
    pub shell: TerminalShell,
    pub working_directory: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalStartResponse {
    pub session_id: String,
    pub shell: TerminalShell,
    pub working_directory: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionInfo {
    pub session_id: String,
    pub shell: TerminalShell,
    pub working_directory: String,
    pub active: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalWriteRequest {
    pub session_id: String,
    pub input: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalResizeRequest {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalStopRequest {
    pub session_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSessionRequest {
    pub session_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TerminalShell {
    PowerShell,
    Cmd,
    Zsh,
    Bash,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputEvent {
    pub session_id: String,
    pub stream: TerminalOutputStream,
    pub data: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TerminalOutputStream {
    Stdout,
    Stderr,
    Exit,
}

impl TerminalShell {
    pub fn program(self) -> &'static str {
        match self {
            TerminalShell::PowerShell => {
                if cfg!(windows) {
                    "powershell.exe"
                } else {
                    "pwsh"
                }
            }
            TerminalShell::Cmd => "cmd.exe",
            TerminalShell::Zsh => "zsh",
            TerminalShell::Bash => "bash",
        }
    }

    pub fn allowed_on_current_platform(self) -> bool {
        if cfg!(windows) {
            matches!(self, TerminalShell::PowerShell | TerminalShell::Cmd)
        } else if cfg!(target_os = "macos") {
            matches!(self, TerminalShell::Zsh | TerminalShell::Bash)
        } else {
            matches!(
                self,
                TerminalShell::Bash | TerminalShell::Zsh | TerminalShell::PowerShell
            )
        }
    }
}
