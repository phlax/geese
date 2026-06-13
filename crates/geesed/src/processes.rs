use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{DateTime, SecondsFormat, Utc};
use thiserror::Error;
use tokio::process::{Child, Command};

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("goose binary unavailable: {0}")]
    GooseBinaryUnavailable(String),
    #[error("spawn failed: {0}")]
    SpawnFailed(String),
    #[error("wait error: {0}")]
    Wait(std::io::Error),
    #[error("signal error: {0}")]
    Signal(nix::errno::Errno),
}

impl From<nix::errno::Errno> for ProcessError {
    fn from(e: nix::errno::Errno) -> Self {
        ProcessError::Signal(e)
    }
}

pub struct GoosedChild {
    pub child: Child,
    pub pid: u32,
    pub started_at: DateTime<Utc>,
}

pub struct RunningProcess {
    pub name: String,
    pub pid: u32,
    pub started_at: String,
}

pub struct ProcessMap {
    children: HashMap<String, GoosedChild>,
    binary: Option<PathBuf>,
}

impl ProcessMap {
    pub fn new(binary: Option<PathBuf>) -> Self {
        Self {
            children: HashMap::new(),
            binary,
        }
    }

    pub async fn start(&mut self, name: &str, profile_path: &Path) -> Result<u32, ProcessError> {
        let binary = self.binary.as_ref().ok_or_else(|| {
            ProcessError::GooseBinaryUnavailable(
                "set GEESE_GOOSE_BIN or install goose on PATH".to_string(),
            )
        })?;

        // Idempotent: return existing pid if already running
        if let Some(child) = self.children.get(name) {
            return Ok(child.pid);
        }

        use std::process::Stdio;
        let mut cmd = Command::new(binary);
        cmd.arg("acp")
            .env("GOOSE_PATH_ROOT", profile_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| ProcessError::SpawnFailed(e.to_string()))?;

        let pid = child
            .id()
            .ok_or_else(|| ProcessError::SpawnFailed("process has no pid".to_string()))?;

        // 100ms startup sanity check: if the binary exits immediately it's broken
        tokio::time::sleep(Duration::from_millis(100)).await;

        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(ProcessError::SpawnFailed(format!(
                    "process exited immediately with status: {status}"
                )));
            }
            Ok(None) => {} // still running
            Err(e) => return Err(ProcessError::SpawnFailed(e.to_string())),
        }

        let started_at = Utc::now();
        self.children.insert(
            name.to_string(),
            GoosedChild {
                child,
                pid,
                started_at,
            },
        );

        Ok(pid)
    }

    pub async fn stop(&mut self, name: &str) -> Result<(), ProcessError> {
        let Some(mut entry) = self.children.remove(name) else {
            return Ok(()); // idempotent: not running is fine
        };

        use nix::errno::Errno;
        match nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(entry.pid as i32),
            nix::sys::signal::Signal::SIGTERM,
        ) {
            Ok(()) => {}
            Err(Errno::ESRCH) => return Ok(()), // already dead
            Err(e) => return Err(ProcessError::Signal(e)),
        }

        match tokio::time::timeout(Duration::from_secs(5), entry.child.wait()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(ProcessError::Wait(e)),
            Err(_timeout) => {
                // Timeout: escalate to SIGKILL
                entry.child.start_kill().ok();
                let _ = entry.child.wait().await;
                Ok(())
            }
        }
    }

    pub async fn kill(&mut self, name: &str) -> Result<(), ProcessError> {
        let Some(mut entry) = self.children.remove(name) else {
            return Ok(()); // idempotent
        };

        entry.child.start_kill().ok();
        let _ = entry.child.wait().await;
        Ok(())
    }

    pub fn list(&self) -> Vec<RunningProcess> {
        let mut result: Vec<RunningProcess> = self
            .children
            .iter()
            .map(|(name, child)| RunningProcess {
                name: name.clone(),
                pid: child.pid,
                started_at: child.started_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            })
            .collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }
}
