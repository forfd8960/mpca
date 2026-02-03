//! Standard shell adapter implementation.
//!
//! This module provides a concrete implementation of the `ShellAdapter` trait
//! using `std::process::Command` to execute shell commands.

use crate::error::{MPCAError, Result};
use crate::tools::shell::{CommandOutput, ShellAdapter};
use std::path::Path;
use std::process::Command;

/// Standard shell adapter using `std::process::Command`.
///
/// This adapter executes real shell commands. For testing, use a mock
/// implementation instead.
#[derive(Debug, Default)]
pub struct StdShellAdapter;

impl StdShellAdapter {
    /// Creates a new standard shell adapter.
    ///
    /// # Returns
    ///
    /// A new `StdShellAdapter` instance.
    pub fn new() -> Self {
        Self
    }

    /// Helper to execute a command and capture output.
    fn execute_command(
        &self,
        cmd: &str,
        cwd: Option<&Path>,
        streaming: bool,
    ) -> Result<CommandOutput> {
        // On Unix, use sh -c; on Windows, use cmd /C
        #[cfg(unix)]
        let (shell, shell_arg) = ("sh", "-c");
        #[cfg(windows)]
        let (shell, shell_arg) = ("cmd", "/C");

        let mut command = Command::new(shell);
        command.arg(shell_arg).arg(cmd);

        if let Some(dir) = cwd {
            command.current_dir(dir);
        }

        // If streaming, inherit stdio; otherwise capture
        if streaming {
            command.stdout(std::process::Stdio::inherit());
            command.stderr(std::process::Stdio::inherit());
        }

        let output = command.output().map_err(|e| {
            MPCAError::ShellCommandFailed(format!("failed to execute command: {}", e))
        })?;

        Ok(CommandOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

impl ShellAdapter for StdShellAdapter {
    fn run(&self, cmd: &str, cwd: Option<&Path>) -> Result<CommandOutput> {
        self.execute_command(cmd, cwd, false)
    }

    fn run_streaming(&self, cmd: &str, cwd: Option<&Path>) -> Result<CommandOutput> {
        self.execute_command(cmd, cwd, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_simple_command() {
        let adapter = StdShellAdapter::new();
        let output = adapter.run("echo hello", None).unwrap();

        assert!(output.success());
        assert_eq!(output.stdout.trim(), "hello");
    }

    #[test]
    fn test_run_with_cwd() {
        let adapter = StdShellAdapter::new();
        let output = adapter.run("pwd", Some(Path::new("/tmp"))).unwrap();

        assert!(output.success());
        assert!(output.stdout.trim().contains("tmp"));
    }

    #[test]
    fn test_run_failing_command() {
        let adapter = StdShellAdapter::new();
        let output = adapter.run("exit 1", None).unwrap();

        assert!(!output.success());
        assert_eq!(output.exit_code, 1);
    }

    #[test]
    fn test_command_output_success() {
        let output = CommandOutput {
            exit_code: 0,
            stdout: "output".to_string(),
            stderr: "".to_string(),
        };

        assert!(output.success());
    }

    #[test]
    fn test_command_output_failure() {
        let output = CommandOutput {
            exit_code: 1,
            stdout: "".to_string(),
            stderr: "error".to_string(),
        };

        assert!(!output.success());
    }
}
