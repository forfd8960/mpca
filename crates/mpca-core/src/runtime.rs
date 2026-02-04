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

/// Runtime trait for MPCA workflow execution.
///
/// Defines the core interface for executing MPCA workflows. This trait allows
/// for alternative runtime implementations and facilitates testing.
pub trait Runtime {
    /// Initializes a repository for MPCA use.
    ///
    /// Creates `.mpca/` and `.trees/` directories, configuration files,
    /// and updates repository documentation.
    fn init_project(&self) -> Result<()>;

    /// Plans a new feature with the given slug.
    ///
    /// Interactively generates feature specifications through conversation
    /// with Claude.
    fn plan_feature(&self, feature_slug: &str) -> Result<()>;

    /// Executes a planned feature.
    ///
    /// Implements the feature according to its specifications in an
    /// isolated git worktree.
    fn run_feature(&self, feature_slug: &str) -> Result<()>;

    /// Sends a chat message to the agent.
    ///
    /// Enables free-form conversation with Claude without committing to
    /// a specific feature workflow.
    fn chat(&self, message: &str) -> Result<String>;
}

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

        // Initialize prompt manager
        let pm = Self::init_prompt_manager(&config)?;

        Ok(Self {
            config,
            pm,
            tools,
            state,
        })
    }

    /// Initializes the prompt manager with template directory resolution.
    ///
    /// Searches for templates in the following order:
    /// 1. User-specified directories in config.prompt_dirs
    /// 2. Bundled templates in the crate's templates directory
    /// 3. Installed location relative to executable
    fn init_prompt_manager(config: &MpcaConfig) -> Result<Option<mpca_pm::PromptManager>> {
        // Try user-specified directories first
        for dir in &config.prompt_dirs {
            if dir.exists() {
                match mpca_pm::PromptManager::new(dir.clone()) {
                    Ok(pm) => return Ok(Some(pm)),
                    Err(_) => continue,
                }
            }
        }

        // Try to find bundled templates relative to crate root
        // This works for development and when running from source
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let template_dir = std::path::PathBuf::from(manifest_dir)
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("crates/mpca-pm/templates"));

            if let Some(dir) = template_dir
                && dir.exists()
                && let Ok(pm) = mpca_pm::PromptManager::new(dir)
            {
                return Ok(Some(pm));
            }
        }

        // Try relative to executable (for installed version)
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            let template_dir = exe_dir
                .parent()
                .map(|p| p.join("share/mpca/templates"))
                .or_else(|| Some(exe_dir.join("templates")));

            if let Some(dir) = template_dir
                && dir.exists()
                && let Ok(pm) = mpca_pm::PromptManager::new(dir)
            {
                return Ok(Some(pm));
            }
        }

        // Prompt manager is optional - workflows can still run without it
        // but template-based prompts won't be available
        tracing::warn!("Prompt manager not initialized - template directory not found");
        Ok(None)
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

    /// Plans a new feature with the given slug.
    ///
    /// # Arguments
    ///
    /// * `feature_slug` - The feature identifier (e.g., "add-caching").
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful planning, or an error if planning fails.
    ///
    /// # Errors
    ///
    /// Returns errors related to feature planning (see `workflows::plan_feature`).
    pub fn plan_feature(&self, feature_slug: &str) -> Result<()> {
        workflows::plan_feature(
            &self.config,
            feature_slug,
            &*self.tools.fs,
            &*self.tools.git,
        )
    }

    /// Executes a feature plan with the given slug.
    ///
    /// # Arguments
    ///
    /// * `feature_slug` - The feature identifier (e.g., "add-caching").
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful execution, or an error if execution fails.
    ///
    /// # Errors
    ///
    /// Returns errors related to feature execution (see `workflows::execute_feature`).
    pub fn run_feature(&self, feature_slug: &str) -> Result<()> {
        workflows::execute_feature(
            &self.config,
            feature_slug,
            &*self.tools.fs,
            &*self.tools.git,
            &*self.tools.shell,
        )
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

impl Runtime for AgentRuntime {
    fn init_project(&self) -> Result<()> {
        workflows::init_project(&self.config, &*self.tools.fs, &*self.tools.git)
    }

    fn plan_feature(&self, feature_slug: &str) -> Result<()> {
        workflows::plan_feature(
            &self.config,
            feature_slug,
            &*self.tools.fs,
            &*self.tools.git,
        )
    }

    fn run_feature(&self, feature_slug: &str) -> Result<()> {
        workflows::execute_feature(
            &self.config,
            feature_slug,
            &*self.tools.fs,
            &*self.tools.git,
            &*self.tools.shell,
        )
    }

    fn chat(&self, _message: &str) -> Result<String> {
        // TODO: Implement in Stage 5
        // For now, return empty string as stub
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_test_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();

        // Create initial commit
        fs::write(dir.join("README.md"), "# Test Repo").unwrap();
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir)
            .env("PRE_COMMIT_ALLOW_NO_CONFIG", "1")
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
        init_test_repo(temp_dir.path());

        let config = MpcaConfig::new(temp_dir.path().to_path_buf());
        let runtime = AgentRuntime::new(config).unwrap();

        // Should succeed (stub implementation)
        let result = runtime.plan_feature("test-feature");
        if let Err(e) = &result {
            eprintln!("Error: {:#}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_feature_stub() {
        let temp_dir = TempDir::new().unwrap();
        init_test_repo(temp_dir.path());

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
