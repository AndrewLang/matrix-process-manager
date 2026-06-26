use crate::command_knowledge::models::NewApplicationRecord;
use crate::models::CommandError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub trait ApplicationScanner: Send + Sync {
    fn scan(&self) -> Result<Vec<NewApplicationRecord>, CommandError>;
}

pub struct InstalledApplicationScanner {
    version_timeout: Duration,
}

impl InstalledApplicationScanner {
    pub fn new() -> Self {
        Self {
            version_timeout: Duration::from_millis(800),
        }
    }

    pub fn with_version_timeout(version_timeout: Duration) -> Self {
        Self { version_timeout }
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<NewApplicationRecord>, CommandError> {
        for directory in self.path_entries() {
            for candidate in self.executable_candidates(&directory)? {
                if self.name_matches(&candidate, name) {
                    return Ok(Some(NewApplicationRecord {
                        name: self.application_name(&candidate),
                        path: self.display_path(&candidate),
                        version: self.read_version(&candidate),
                    }));
                }
            }
        }

        Ok(None)
    }

    fn path_entries(&self) -> Vec<PathBuf> {
        std::env::var_os("PATH")
            .map(|path| std::env::split_paths(&path).collect())
            .unwrap_or_default()
    }

    fn executable_candidates(&self, directory: &Path) -> Result<Vec<PathBuf>, CommandError> {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_) => return Ok(Vec::new()),
        };

        let mut candidates = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| CommandError::terminal_failed(error.to_string()))?;
            let path = entry.path();
            if self.is_executable(&path) {
                candidates.push(path);
            }
        }
        Ok(candidates)
    }

    fn is_executable(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        #[cfg(windows)]
        {
            let extensions = self.executable_extensions();
            path.extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| {
                    extensions.contains(&format!(".{}", extension).to_ascii_uppercase())
                })
                .unwrap_or(false)
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            path.metadata()
                .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }

        #[cfg(not(any(windows, unix)))]
        {
            true
        }
    }

    #[cfg(windows)]
    fn executable_extensions(&self) -> HashSet<String> {
        std::env::var_os("PATHEXT")
            .map(|value| {
                value
                    .to_string_lossy()
                    .split(';')
                    .filter_map(|extension| {
                        let extension = extension.trim();
                        (!extension.is_empty()).then(|| extension.to_ascii_uppercase())
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                [".COM", ".EXE", ".BAT", ".CMD"]
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            })
    }

    fn normalize_path(&self, path: &Path) -> String {
        let normalized = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .to_string();

        if cfg!(windows) {
            normalized.to_ascii_lowercase()
        } else {
            normalized
        }
    }

    fn display_path(&self, path: &Path) -> String {
        path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .to_string()
    }

    fn application_name(&self, path: &Path) -> String {
        if cfg!(windows) {
            path.file_stem()
                .or_else(|| path.file_name())
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| self.display_path(path))
        } else {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| self.display_path(path))
        }
    }

    fn name_matches(&self, path: &Path, name: &str) -> bool {
        self.application_name(path).eq_ignore_ascii_case(name)
            || path
                .file_name()
                .map(|file_name| file_name.to_string_lossy().eq_ignore_ascii_case(name))
                .unwrap_or(false)
    }

    fn read_version(&self, path: &Path) -> Option<String> {
        let mut child = Command::new(path)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .ok()?;

        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if start.elapsed() >= self.version_timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                Ok(None) => thread::sleep(Duration::from_millis(20)),
                Err(_) => return None,
            }
        }

        child.wait_with_output().ok().and_then(|output| {
            let mut data = String::new();
            data.push_str(&String::from_utf8_lossy(&output.stdout));
            if data.trim().is_empty() {
                data.push_str(&String::from_utf8_lossy(&output.stderr));
            }
            data.lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(|line| line.chars().take(512).collect())
        })
    }
}

impl ApplicationScanner for InstalledApplicationScanner {
    fn scan(&self) -> Result<Vec<NewApplicationRecord>, CommandError> {
        let mut seen = HashSet::new();
        let mut records = Vec::new();

        for directory in self.path_entries() {
            for candidate in self.executable_candidates(&directory)? {
                let normalized = self.normalize_path(&candidate);
                if seen.insert(normalized) {
                    records.push(NewApplicationRecord {
                        name: self.application_name(&candidate),
                        path: self.display_path(&candidate),
                        version: self.read_version(&candidate),
                    });
                }
            }
        }

        Ok(records)
    }
}

impl Default for InstalledApplicationScanner {
    fn default() -> Self {
        Self::new()
    }
}
