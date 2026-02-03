//! Tool adapters and registry for MPCA workflows.
//!
//! This module provides the tool registry that manages different adapters
//! for file system, git, and shell operations. Each adapter trait defines
//! the interface for a specific category of operations.

pub mod fs;
pub mod fs_impl;
pub mod git;
pub mod git_impl;
pub mod shell;
pub mod shell_impl;

/// Tool registry that manages all available adapters.
///
/// The registry owns instances of each adapter (file system, git, shell)
/// and provides access to them during workflow execution. Adapters are
/// trait objects to allow for different implementations (e.g., real vs. mock).
pub struct ToolRegistry {
    /// File system adapter for read/write operations.
    pub fs: Box<dyn fs::FsAdapter>,

    /// Git adapter for repository operations.
    pub git: Box<dyn git::GitAdapter>,

    /// Shell adapter for command execution.
    pub shell: Box<dyn shell::ShellAdapter>,
}

impl ToolRegistry {
    /// Creates a new tool registry with the provided adapters.
    ///
    /// # Arguments
    ///
    /// * `fs` - File system adapter implementation.
    /// * `git` - Git adapter implementation.
    /// * `shell` - Shell adapter implementation.
    ///
    /// # Returns
    ///
    /// A new `ToolRegistry` containing the provided adapters.
    pub fn new(
        fs: Box<dyn fs::FsAdapter>,
        git: Box<dyn git::GitAdapter>,
        shell: Box<dyn shell::ShellAdapter>,
    ) -> Self {
        Self { fs, git, shell }
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("fs", &"Box<dyn FsAdapter>")
            .field("git", &"Box<dyn GitAdapter>")
            .field("shell", &"Box<dyn ShellAdapter>")
            .finish()
    }
}
