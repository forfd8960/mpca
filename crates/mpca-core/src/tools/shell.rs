//! Shell adapter trait and operations.
//!
//! This module defines the `ShellAdapter` trait for executing shell commands,
//! allowing for both real command execution and mock implementations for testing.

use crate::error::Result;
use std::path::Path;

/// Shell command output.
///
/// Contains the result of a shell command execution, including exit code,
/// stdout, and stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Exit code from the command (0 typically indicates success).
    pub exit_code: i32,

    /// Standard output from the command.
    pub stdout: String,

    /// Standard error output from the command.
    pub stderr: String,
}

impl CommandOutput {
    /// Checks if the command succeeded (exit code 0).
    ///
    /// # Returns
    ///
    /// `true` if the exit code is 0, `false` otherwise.
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Shell adapter trait.
///
/// Defines the interface for executing shell commands needed by MPCA workflows.
/// Implementations can execute real commands or provide mocked behavior for testing.
pub trait ShellAdapter: Send + Sync {
    /// Executes a shell command and waits for completion.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to execute (including arguments).
    /// * `cwd` - Working directory for the command (optional).
    ///
    /// # Returns
    ///
    /// The command output including exit code, stdout, and stderr,
    /// or an error if the command cannot be executed.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::ShellCommandFailed` if the command fails to execute
    /// (not if it returns a non-zero exit code - check `CommandOutput::success()` for that),
    /// or `MPCAError::Io` for IO errors.
    fn run(&self, cmd: &str, cwd: Option<&Path>) -> Result<CommandOutput>;

    /// Executes a shell command and streams output.
    ///
    /// This method is intended for long-running commands where output should
    /// be displayed to the user in real-time (e.g., test execution, builds).
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to execute (including arguments).
    /// * `cwd` - Working directory for the command (optional).
    ///
    /// # Returns
    ///
    /// The final command output after completion, or an error if the command
    /// cannot be executed.
    ///
    /// # Errors
    ///
    /// Returns `MPCAError::ShellCommandFailed` if the command fails to execute,
    /// or `MPCAError::Io` for IO errors.
    fn run_streaming(&self, cmd: &str, cwd: Option<&Path>) -> Result<CommandOutput>;
}
