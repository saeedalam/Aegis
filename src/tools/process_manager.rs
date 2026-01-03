//! Process manager for safe subprocess execution.
//!
//! Provides timeout-aware process spawning and monitoring.

use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::tools::ToolError;

/// Default timeout for tool execution (30 seconds).
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Process manager for executing subprocesses safely.
#[derive(Debug, Clone)]
pub struct ProcessManager {
    /// Maximum execution time before killing the process.
    timeout_secs: u64,
}

impl ProcessManager {
    /// Creates a new process manager with the default timeout.
    pub fn new() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    /// Creates a new process manager with a custom timeout.
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }

    /// Executes a command and returns the output.
    ///
    /// The process is killed if it exceeds the configured timeout.
    pub async fn execute(
        &self,
        program: &str,
        args: &[&str],
    ) -> Result<ProcessOutput, ToolError> {
        debug!("Executing: {} {:?}", program, args);

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true) // Kill the process if the future is dropped
            .spawn()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn process: {}", e)))?;

        let timeout_duration = Duration::from_secs(self.timeout_secs);

        // Wait for the process with timeout
        match timeout(timeout_duration, child.wait()).await {
            Ok(exit_result) => {
                let status = exit_result.map_err(|e| {
                    ToolError::ExecutionFailed(format!("Process error: {}", e))
                })?;

                // Read stdout and stderr after process exits
                let stdout = if let Some(mut stdout) = child.stdout.take() {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    stdout.read_to_end(&mut buf).await.unwrap_or_default();
                    String::from_utf8_lossy(&buf).to_string()
                } else {
                    String::new()
                };

                let stderr = if let Some(mut stderr) = child.stderr.take() {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    stderr.read_to_end(&mut buf).await.unwrap_or_default();
                    String::from_utf8_lossy(&buf).to_string()
                } else {
                    String::new()
                };

                let exit_code = status.code().unwrap_or(-1);

                debug!("Process completed with exit code: {}", exit_code);

                Ok(ProcessOutput {
                    stdout,
                    stderr,
                    exit_code,
                    success: status.success(),
                })
            }
            Err(_) => {
                // Timeout - process will be killed by kill_on_drop
                warn!("Process timed out after {}s", self.timeout_secs);
                Err(ToolError::Timeout(self.timeout_secs))
            }
        }
    }

    /// Executes a shell command (via sh -c).
    pub async fn execute_shell(&self, command: &str) -> Result<ProcessOutput, ToolError> {
        self.execute("sh", &["-c", command]).await
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from a process execution.
#[derive(Debug, Clone)]
pub struct ProcessOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code (-1 if terminated by signal).
    pub exit_code: i32,
    /// Whether the process exited successfully.
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_echo() {
        let pm = ProcessManager::new();
        let result = pm.execute("echo", &["hello"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.stdout.trim() == "hello");
    }

    #[tokio::test]
    async fn test_execute_timeout() {
        let pm = ProcessManager::with_timeout(1);
        let result = pm.execute("sleep", &["10"]).await;
        assert!(matches!(result, Err(ToolError::Timeout(_))));
    }
}
