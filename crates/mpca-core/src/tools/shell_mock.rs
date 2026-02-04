//! Mock shell adapter for testing.
//!
//! This module provides a mock implementation of the `ShellAdapter` trait
//! for use in tests. The mock allows predefined command outputs and
//! tracks executed commands.

use crate::error::{MPCAError, Result};
use crate::tools::shell::{CommandOutput, ShellAdapter};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Type alias for command history entry (command, working_directory)
type CommandHistoryEntry = (String, Option<PathBuf>);

/// Mock shell adapter for testing.
///
/// Allows pre-programming command outputs and tracking command execution.
/// Useful for testing workflows without executing real shell commands.
///
/// # Examples
///
/// ```
/// use mpca_core::tools::shell_mock::MockShellAdapter;
/// use mpca_core::tools::shell::{ShellAdapter, CommandOutput};
/// use std::path::Path;
///
/// let shell = MockShellAdapter::new();
/// shell.set_output(
///     "cargo test",
///     CommandOutput {
///         exit_code: 0,
///         stdout: "test result: ok. 5 passed".to_string(),
///         stderr: String::new(),
///     }
/// );
///
/// let output = shell.run("cargo test", None).unwrap();
/// assert_eq!(output.exit_code, 0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct MockShellAdapter {
    /// Pre-programmed command outputs (command -> output)
    outputs: Arc<Mutex<HashMap<String, CommandOutput>>>,
    /// History of executed commands
    history: Arc<Mutex<Vec<CommandHistoryEntry>>>,
    /// Default output for unknown commands
    default_output: Arc<Mutex<Option<CommandOutput>>>,
}

impl MockShellAdapter {
    /// Creates a new mock shell adapter.
    ///
    /// # Returns
    ///
    /// A new `MockShellAdapter` with no pre-programmed outputs.
    pub fn new() -> Self {
        Self {
            outputs: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(Vec::new())),
            default_output: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a mock with success as default response.
    ///
    /// # Returns
    ///
    /// A `MockShellAdapter` that returns success for all commands.
    pub fn with_success() -> Self {
        let adapter = Self::new();
        adapter.set_default_output(CommandOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        });
        adapter
    }

    /// Sets the output for a specific command.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command string to match
    /// * `output` - Output to return when command is executed
    ///
    /// # Examples
    ///
    /// ```
    /// use mpca_core::tools::shell_mock::MockShellAdapter;
    /// use mpca_core::tools::shell::CommandOutput;
    ///
    /// let shell = MockShellAdapter::new();
    /// shell.set_output("ls", CommandOutput {
    ///     exit_code: 0,
    ///     stdout: "file1.txt\nfile2.txt".to_string(),
    ///     stderr: String::new(),
    /// });
    /// ```
    pub fn set_output(&self, cmd: &str, output: CommandOutput) {
        self.outputs.lock().unwrap().insert(cmd.to_string(), output);
    }

    /// Sets the default output for unknown commands.
    ///
    /// # Arguments
    ///
    /// * `output` - Default output to use
    pub fn set_default_output(&self, output: CommandOutput) {
        *self.default_output.lock().unwrap() = Some(output);
    }

    /// Returns the history of executed commands.
    ///
    /// # Returns
    ///
    /// Vector of (command, working_directory) tuples.
    pub fn get_history(&self) -> Vec<CommandHistoryEntry> {
        self.history.lock().unwrap().clone()
    }

    /// Returns the number of times a command was executed.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to count
    ///
    /// # Returns
    ///
    /// Number of times the command was executed.
    pub fn command_count(&self, cmd: &str) -> usize {
        self.history
            .lock()
            .unwrap()
            .iter()
            .filter(|(c, _)| c == cmd)
            .count()
    }

    /// Clears command history.
    pub fn clear_history(&self) {
        self.history.lock().unwrap().clear();
    }

    /// Clears all outputs and history.
    pub fn clear(&self) {
        self.outputs.lock().unwrap().clear();
        self.history.lock().unwrap().clear();
        *self.default_output.lock().unwrap() = None;
    }
}

impl ShellAdapter for MockShellAdapter {
    fn run(&self, cmd: &str, cwd: Option<&Path>) -> Result<CommandOutput> {
        // Record command in history
        self.history
            .lock()
            .unwrap()
            .push((cmd.to_string(), cwd.map(|p| p.to_path_buf())));

        // Return pre-programmed output or default
        let outputs = self.outputs.lock().unwrap();
        if let Some(output) = outputs.get(cmd) {
            Ok(output.clone())
        } else if let Some(default) = self.default_output.lock().unwrap().clone() {
            Ok(default)
        } else {
            Err(MPCAError::ShellCommandFailed(format!(
                "No output configured for command: {}",
                cmd
            )))
        }
    }

    fn run_streaming(&self, cmd: &str, cwd: Option<&Path>) -> Result<CommandOutput> {
        // For mock, streaming is same as regular run
        self.run(cmd, cwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_shell_basic() {
        let shell = MockShellAdapter::new();
        shell.set_output(
            "echo hello",
            CommandOutput {
                exit_code: 0,
                stdout: "hello\n".to_string(),
                stderr: String::new(),
            },
        );

        let output = shell.run("echo hello", None).unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, "hello\n");
    }

    #[test]
    fn test_mock_shell_with_success() {
        let shell = MockShellAdapter::with_success();

        let output = shell.run("any command", None).unwrap();
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn test_mock_shell_command_not_found() {
        let shell = MockShellAdapter::new();

        let result = shell.run("unknown command", None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MPCAError::ShellCommandFailed(_)
        ));
    }

    #[test]
    fn test_mock_shell_history() {
        let shell = MockShellAdapter::with_success();

        shell.run("cmd1", None).unwrap();
        shell.run("cmd2", Some(Path::new("/dir"))).unwrap();
        shell.run("cmd1", None).unwrap();

        let history = shell.get_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].0, "cmd1");
        assert_eq!(history[1].0, "cmd2");
        assert_eq!(history[1].1, Some(PathBuf::from("/dir")));
        assert_eq!(history[2].0, "cmd1");
    }

    #[test]
    fn test_mock_shell_command_count() {
        let shell = MockShellAdapter::with_success();

        shell.run("cmd1", None).unwrap();
        shell.run("cmd2", None).unwrap();
        shell.run("cmd1", None).unwrap();

        assert_eq!(shell.command_count("cmd1"), 2);
        assert_eq!(shell.command_count("cmd2"), 1);
        assert_eq!(shell.command_count("cmd3"), 0);
    }

    #[test]
    fn test_mock_shell_clear_history() {
        let shell = MockShellAdapter::with_success();

        shell.run("cmd", None).unwrap();
        assert_eq!(shell.get_history().len(), 1);

        shell.clear_history();
        assert_eq!(shell.get_history().len(), 0);
    }

    #[test]
    fn test_mock_shell_clear() {
        let shell = MockShellAdapter::new();
        shell.set_output(
            "cmd",
            CommandOutput {
                exit_code: 0,
                stdout: "output".to_string(),
                stderr: String::new(),
            },
        );
        shell.run("cmd", None).unwrap();

        shell.clear();

        assert_eq!(shell.get_history().len(), 0);
        assert!(shell.run("cmd", None).is_err());
    }

    #[test]
    fn test_mock_shell_streaming() {
        let shell = MockShellAdapter::new();
        shell.set_output(
            "cargo test",
            CommandOutput {
                exit_code: 0,
                stdout: "test result: ok".to_string(),
                stderr: String::new(),
            },
        );

        let output = shell.run_streaming("cargo test", None).unwrap();
        assert_eq!(output.stdout, "test result: ok");
    }

    #[test]
    fn test_mock_shell_failure_output() {
        let shell = MockShellAdapter::new();
        shell.set_output(
            "failing cmd",
            CommandOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: "error message".to_string(),
            },
        );

        let output = shell.run("failing cmd", None).unwrap();
        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stderr, "error message");
    }
}
