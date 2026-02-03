//! Runtime for MPCA agent workflows.
//!
//! This module provides the `AgentRuntime` struct which orchestrates all MPCA
//! workflows, manages state, and coordinates between the prompt manager, tools,
//! and the Claude Agent SDK.

use crate::config::MpcaConfig;
use crate::error::Result;
use crate::state::RuntimeState;
use crate::tools::ToolRegistry;
use crate::tools::fs_impl::StdFsAdapter;
use crate::tools::git_impl::StdGitAdapter;
use crate::tools::shell_impl::StdShellAdapter;
use crate::workflows;

/// Agent runtime for MPCA workflows.
///
/// The runtime is the main entry point for executing MPCA workflows. It manages
/// configuration, state, tools, and coordinates with the prompt manager and
/// Claude Agent SDK.
///
/// # Examples
///
/// ```no_run
/// use mpca_core::{AgentRuntime, MpcaConfig};
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = MpcaConfig::new(PathBuf::from("/path/to/repo"));
/// let runtime = AgentRuntime::new(config)?;
///
/// // Initialize repository
/// runtime.init_project()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct AgentRuntime {
    /// MPCA configuration.
    pub config: MpcaConfig,

    /// Prompt manager for template rendering.
    pub pm: Option<mpca_pm::PromptManager>,

    /// Tool registry for file system, git, and shell operations.
    pub tools: ToolRegistry,

    /// Runtime state tracking workflow progress.
    pub state: RuntimeState,
}

impl AgentRuntime {
    /// Creates a new agent runtime with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - MPCA configuration containing repository paths and settings.
    ///
    /// # Returns
    ///
    /// A new `AgentRuntime` instance ready to execute workflows.
    ///
    /// # Errors
    ///
    /// Returns an error if tool initialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mpca_core::{AgentRuntime, MpcaConfig};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MpcaConfig::new(PathBuf::from("/path/to/repo"));
    /// let runtime = AgentRuntime::new(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: MpcaConfig) -> Result<Self> {
        // Create tool registry with standard implementations
        let tools = ToolRegistry::new(
            Box::new(StdFsAdapter::new()),
            Box::new(StdGitAdapter::new()),
            Box::new(StdShellAdapter::new()),
        );

        // Initialize runtime state
        let state = RuntimeState::default();

        Ok(Self {
            config,
            pm: None,
            tools,
            state,
        })
    }

    /// Initializes a repository for MPCA use.
    ///
    /// This workflow:
    /// 1. Verifies the directory is a git repository
    /// 2. Creates `.mpca/` and `.trees/` directories
    /// 3. Creates default configuration file
    /// 4. Updates `.gitignore` to exclude `.trees/`
    /// 5. Creates or updates `CLAUDE.md` with MPCA documentation
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if initialization fails.
    ///
    /// # Errors
    ///
    /// Returns:
    /// - `MPCAError::NotGitRepository` if not in a git repository
    /// - `MPCAError::AlreadyInitialized` if already initialized
    /// - `MPCAError::FileWriteError` if file creation fails
    /// - `MPCAError::PermissionDenied` if lacking write permissions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mpca_core::{AgentRuntime, MpcaConfig};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = MpcaConfig::new(PathBuf::from("/path/to/repo"));
    /// let runtime = AgentRuntime::new(config)?;
    ///
    /// runtime.init_project()?;
    /// println!("Repository initialized for MPCA!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn init_project(&self) -> Result<()> {
        workflows::init_project(&self.config, &*self.tools.fs, &*self.tools.git)
    }

    /// Plans a new feature with the given slug (to be implemented in Stage 4).
    ///
    /// # Arguments
    ///
    /// * `_feature_slug` - The feature identifier (e.g., "add-caching").
    ///
    /// # Returns
    ///
    /// Currently returns `Ok(())` as a stub. Will be fully implemented in Stage 4.
    ///
    /// # Errors
    ///
    /// Will return errors related to feature planning once implemented.
    #[allow(unused_variables)]
    pub fn plan_feature(&self, _feature_slug: &str) -> Result<()> {
        // TODO: Implement in Stage 4
        // - Load prompt manager
        // - Initialize Claude agent with plan mode
        // - Run interactive TUI for feature planning
        // - Save specs to .mpca/specs/<feature-slug>/
        // - Create git worktree in .trees/<feature-slug>/
        Ok(())
    }

    /// Executes a feature plan with the given slug (to be implemented in Stage 4).
    ///
    /// # Arguments
    ///
    /// * `_feature_slug` - The feature identifier (e.g., "add-caching").
    ///
    /// # Returns
    ///
    /// Currently returns `Ok(())` as a stub. Will be fully implemented in Stage 4.
    ///
    /// # Errors
    ///
    /// Will return errors related to feature execution once implemented.
    #[allow(unused_variables)]
    pub fn run_feature(&self, _feature_slug: &str) -> Result<()> {
        // TODO: Implement in Stage 4
        // - Load feature specs from .mpca/specs/<feature-slug>/
        // - Initialize Claude agent with run mode
        // - Execute implementation workflow
        // - Run verification
        // - Commit changes
        // - Generate report
        Ok(())
    }

    /// Sends a chat message to the agent (to be implemented in Stage 4).
    ///
    /// # Arguments
    ///
    /// * `_message` - The message to send to the agent.
    ///
    /// # Returns
    ///
    /// Currently returns an empty string as a stub. Will return the agent's
    /// response once implemented in Stage 4.
    ///
    /// # Errors
    ///
    /// Will return errors related to agent communication once implemented.
    #[allow(unused_variables)]
    pub fn chat(&self, _message: &str) -> Result<String> {
        // TODO: Implement in Stage 4
        // - Initialize Claude agent if not already done
        // - Send message to agent
        // - Return response
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_test_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn test_new_runtime() {
        let temp_dir = TempDir::new().unwrap();
        let config = MpcaConfig::new(temp_dir.path().to_path_buf());

        let runtime = AgentRuntime::new(config);
        assert!(runtime.is_ok());

        let runtime = runtime.unwrap();
        assert!(runtime.pm.is_none());
        assert_eq!(runtime.state.feature_slug, None);
    }

    #[test]
    fn test_init_project_integration() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let runtime = AgentRuntime::new(config).unwrap();

        let result = runtime.init_project();
        assert!(result.is_ok());

        // Verify initialization artifacts
        assert!(runtime.tools.fs.exists(&runtime.config.specs_dir));
        assert!(runtime.tools.fs.exists(&runtime.config.trees_dir));
        assert!(runtime.tools.fs.exists(&runtime.config.config_file));
        assert!(runtime.tools.fs.exists(&runtime.config.claude_md));
    }

    #[test]
    fn test_plan_feature_stub() {
        let temp_dir = TempDir::new().unwrap();
        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let runtime = AgentRuntime::new(config).unwrap();

        // Should succeed (stub implementation)
        let result = runtime.plan_feature("test-feature");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_feature_stub() {
        let temp_dir = TempDir::new().unwrap();
        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let runtime = AgentRuntime::new(config).unwrap();

        // Should succeed (stub implementation)
        let result = runtime.run_feature("test-feature");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chat_stub() {
        let temp_dir = TempDir::new().unwrap();
        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let runtime = AgentRuntime::new(config).unwrap();

        // Should succeed (stub implementation)
        let result = runtime.chat("Hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
