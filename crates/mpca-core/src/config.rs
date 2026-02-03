//! Configuration types for MPCA runtime.
//!
//! This module defines all configuration structures used throughout MPCA,
//! including main configuration, git settings, review settings, agent modes,
//! and tool sets.

use std::path::PathBuf;

/// Main MPCA configuration.
///
/// Contains all paths, settings, and sub-configurations needed for MPCA runtime.
/// This structure is typically loaded from `.mpca/config.toml` with defaults
/// applied for missing values.
#[derive(Debug, Clone)]
pub struct MpcaConfig {
    /// Repository root directory (absolute path).
    pub repo_root: PathBuf,

    /// Directory for git worktrees (typically `.trees`).
    pub trees_dir: PathBuf,

    /// Directory for feature specs (typically `.mpca/specs`).
    pub specs_dir: PathBuf,

    /// Path to CLAUDE.md file in repository root.
    pub claude_md: PathBuf,

    /// Path to MPCA configuration file (`.mpca/config.toml`).
    pub config_file: PathBuf,

    /// Additional prompt template directories (for user overrides).
    pub prompt_dirs: Vec<PathBuf>,

    /// Git-related configuration.
    pub git: GitConfig,

    /// Code review configuration.
    pub review: ReviewConfig,

    /// Agent mode configuration per workflow.
    pub agent_modes: WorkflowModes,

    /// Tool set configuration per workflow.
    pub tool_sets: WorkflowTools,
}

impl MpcaConfig {
    /// Creates a new configuration with sensible defaults.
    ///
    /// # Arguments
    ///
    /// * `repo_root` - The repository root directory (must be absolute path).
    ///
    /// # Returns
    ///
    /// A new `MpcaConfig` with all paths derived from `repo_root` and default
    /// settings for git, review, agent modes, and tool sets.
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            trees_dir: repo_root.join(".trees"),
            specs_dir: repo_root.join(".mpca").join("specs"),
            claude_md: repo_root.join("CLAUDE.md"),
            config_file: repo_root.join(".mpca").join("config.toml"),
            prompt_dirs: Vec::new(),
            repo_root,
            git: GitConfig::default(),
            review: ReviewConfig::default(),
            agent_modes: WorkflowModes::default(),
            tool_sets: WorkflowTools::default(),
        }
    }
}

/// Git-related configuration.
///
/// Controls git behavior for MPCA workflows, including automatic commits
/// and branch naming conventions.
#[derive(Debug, Clone)]
pub struct GitConfig {
    /// Whether to automatically commit changes during workflows.
    pub auto_commit: bool,

    /// Branch naming pattern (can include placeholders like `{feature_slug}`).
    pub branch_naming: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            auto_commit: true,
            branch_naming: "feature/{feature_slug}".to_string(),
        }
    }
}

/// Code review configuration.
///
/// Controls code review behavior, including whether reviews are enabled
/// and the list of reviewers.
#[derive(Debug, Clone, Default)]
pub struct ReviewConfig {
    /// Whether code review is enabled for this repository.
    pub enabled: bool,

    /// List of reviewers (usernames or email addresses).
    pub reviewers: Vec<String>,
}

/// Agent mode configuration for a specific workflow.
///
/// Defines how the Claude agent should behave for a particular workflow,
/// including model selection, temperature, and whether to use code presets.
#[derive(Debug, Clone)]
pub struct AgentMode {
    /// Whether to use Claude Code preset (enables advanced code understanding).
    pub use_code_preset: bool,

    /// Model identifier (e.g., "claude-3-5-sonnet-20241022").
    pub model: String,

    /// Temperature for generation (0.0 = deterministic, 1.0 = creative).
    pub temperature: f32,

    /// Maximum tokens for response.
    pub max_tokens: u32,
}

/// Agent mode configuration for all workflows.
///
/// Provides defaults for each workflow type with appropriate settings.
/// Users can override these in `.mpca/config.toml`.
#[derive(Debug, Clone)]
pub struct WorkflowModes {
    /// Agent mode for init workflow.
    pub init: AgentMode,

    /// Agent mode for plan workflow.
    pub plan: AgentMode,

    /// Agent mode for execute workflow.
    pub execute: AgentMode,

    /// Agent mode for review workflow.
    pub review: AgentMode,

    /// Agent mode for verify workflow.
    pub verify: AgentMode,
}

impl Default for WorkflowModes {
    fn default() -> Self {
        Self {
            init: AgentMode {
                use_code_preset: false,
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: 0.0,
                max_tokens: 4096,
            },
            plan: AgentMode {
                use_code_preset: true,
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: 0.3,
                max_tokens: 8192,
            },
            execute: AgentMode {
                use_code_preset: true,
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: 0.0,
                max_tokens: 8192,
            },
            review: AgentMode {
                use_code_preset: true,
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: 0.0,
                max_tokens: 8192,
            },
            verify: AgentMode {
                use_code_preset: false,
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: 0.0,
                max_tokens: 4096,
            },
        }
    }
}

/// Tool set variants for different workflow needs.
///
/// Defines the level of tool access granted to the agent for a workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSet {
    /// Minimal tools: fs (read), git (status).
    Minimal,

    /// Standard tools: fs (read/write), git (status/commit), shell (limited).
    Standard,

    /// Full tools: fs (full), git (full), shell (full), test_runner, search.
    Full,
}

/// Tool set configuration for all workflows.
///
/// Defines which tools are available to each workflow type.
/// Follows principle of least privilege by default.
#[derive(Debug, Clone)]
pub struct WorkflowTools {
    /// Tool set for init workflow.
    pub init: ToolSet,

    /// Tool set for plan workflow.
    pub plan: ToolSet,

    /// Tool set for execute workflow.
    pub execute: ToolSet,

    /// Tool set for review workflow.
    pub review: ToolSet,

    /// Tool set for verify workflow.
    pub verify: ToolSet,
}

impl Default for WorkflowTools {
    fn default() -> Self {
        Self {
            init: ToolSet::Minimal,
            plan: ToolSet::Standard,
            execute: ToolSet::Full,
            review: ToolSet::Standard,
            verify: ToolSet::Standard,
        }
    }
}
