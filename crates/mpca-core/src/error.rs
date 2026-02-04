//! Error types for MPCA operations.
//!
//! This module defines all error variants that can occur during MPCA workflows,
//! from initialization to verification. All errors use `thiserror` for ergonomic
//! error handling with context.

use std::path::PathBuf;
use thiserror::Error;

/// Comprehensive error types for MPCA operations.
///
/// Each variant represents a specific failure mode with relevant context,
/// enabling precise error handling and user-friendly error messages.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum MPCAError {
    // Initialization errors
    /// Not a git repository at the specified path.
    #[error("not a git repository: {0}")]
    NotGitRepository(PathBuf),

    /// Repository already initialized by MPCA.
    #[error("repo already initialized by MPCA")]
    AlreadyInitialized,

    /// Repository not initialized by MPCA - user needs to run `mpca init` first.
    #[error("repo not initialized by MPCA - run `mpca init` first")]
    NotInitialized,

    /// Permission denied for the specified operation.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    // Feature errors
    /// Feature with the given slug was not found.
    #[error("feature not found: {0}")]
    FeatureNotFound(String),

    /// Feature with the given slug already exists.
    #[error("feature already exists: {0}")]
    FeatureAlreadyExists(String),

    /// Invalid feature slug format (must be lowercase alphanumeric with hyphens).
    #[error("invalid feature slug: {0} (must be lowercase alphanumeric with hyphens)")]
    InvalidFeatureSlug(String),

    // State errors
    /// State file is corrupted and cannot be parsed.
    #[error("corrupted state file: {0}")]
    CorruptedState(PathBuf),

    /// Invalid state transition attempted.
    #[error("invalid state transition from {0} to {1}")]
    InvalidStateTransition(String, String),

    /// State file is missing at the expected location.
    #[error("state file missing: {0}")]
    StateMissing(PathBuf),

    // Git errors
    /// Git worktree already exists at the specified path.
    #[error("worktree already exists: {0}")]
    WorktreeExists(PathBuf),

    /// Git branch already exists with the specified name.
    #[error("branch already exists: {0}")]
    BranchExists(String),

    /// Uncommitted changes exist in the worktree.
    #[error("uncommitted changes in worktree: {0}")]
    UncommittedChanges(PathBuf),

    /// Git command failed with the specified error.
    #[error("git command failed: {0}")]
    GitCommandFailed(String),

    /// Git worktree not found at the specified path.
    #[error("worktree not found: {0}")]
    WorktreeNotFound(PathBuf),

    // File system errors
    /// Path not found in the file system.
    #[error("path not found: {0}")]
    PathNotFound(PathBuf),

    /// Invalid path provided.
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),

    /// Error reading file.
    #[error("file read error: {0}")]
    FileReadError(String),

    /// Error writing file.
    #[error("file write error: {0}")]
    FileWriteError(String),

    // Config errors
    /// Invalid configuration detected.
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// Error parsing configuration file.
    #[error("config parse error: {0}")]
    ConfigParseError(String),

    /// Required configuration field is missing.
    #[error("missing required config field: {0}")]
    MissingConfigField(String),

    /// Configuration file not found at the expected location.
    #[error("config file not found: {0}")]
    ConfigNotFound(PathBuf),

    // Prompt/template errors
    /// Template not found with the specified name.
    #[error("template not found: {0}")]
    TemplateNotFound(String),

    /// Error rendering template.
    #[error("template render error: {0}")]
    TemplateRenderError(String),

    /// Invalid template context provided.
    #[error("invalid template context: {0}")]
    InvalidTemplateContext(String),

    // Agent/SDK errors
    /// Claude agent SDK error occurred.
    #[error("claude agent error: {0}")]
    AgentError(String),

    /// API authentication failed.
    #[error("API authentication failed")]
    AuthenticationFailed,

    /// API rate limit exceeded.
    #[error("API rate limit exceeded")]
    RateLimitExceeded,

    /// Agent operation timed out.
    #[error("agent timeout after {0}s")]
    AgentTimeout(u64),

    // Plan errors
    /// Invalid plan format detected.
    #[error("invalid plan format: {0}")]
    InvalidPlanFormat(String),

    /// Plan validation failed.
    #[error("plan validation failed: {0}")]
    PlanValidationFailed(String),

    /// Required plan section is missing.
    #[error("missing plan section: {0}")]
    MissingPlanSection(String),

    /// Plan not found for the specified feature.
    #[error("plan not found for feature: {0}")]
    PlanNotFound(String),

    // Verification errors
    /// Verification failed with the specified error.
    #[error("verification failed: {0}")]
    VerificationFailed(String),

    /// Tests failed with the specified error.
    #[error("tests failed: {0}")]
    TestsFailed(String),

    /// Verification spec missing for the specified feature.
    #[error("verification spec missing for feature: {0}")]
    VerificationSpecMissing(String),

    /// Verification operation timed out.
    #[error("verification timeout after {0}s")]
    VerificationTimeout(u64),

    // Tool/adapter errors
    /// Shell command failed with the specified error.
    #[error("shell command failed: {0}")]
    ShellCommandFailed(String),

    /// Tool execution error occurred.
    #[error("tool execution error: {0}")]
    ToolExecutionError(String),

    // IO and system errors
    /// Standard IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    // Anyhow passthrough for rich context
    /// Generic error with context from anyhow.
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    // Generic fallback
    /// Unexpected error occurred.
    #[error("unexpected error: {0}")]
    Other(String),
}

/// Result type alias for MPCA operations.
///
/// All fallible MPCA operations return this type, using [`MPCAError`] for error variants.
pub type Result<T> = std::result::Result<T, MPCAError>;
