//! MPCA Core - Execution engine for Mine Personal Coding Agent.
//!
//! This crate provides the core execution engine for MPCA workflows,
//! including runtime state management, configuration, error handling,
//! and tool adapters for file system, git, and shell operations.
//!
//! # Architecture
//!
//! The core crate is organized into several modules:
//!
//! - [`error`]: Error types and result type alias
//! - [`config`]: Configuration structures for MPCA runtime
//! - [`state`]: Runtime state and workflow phase tracking
//! - [`tools`]: Tool registry and adapter traits
//! - [`runtime`]: Agent runtime for orchestrating workflows
//! - [`workflows`]: Workflow implementations (init, plan, run, verify)
//!
//! # Example
//!
//! ```rust,ignore
//! use mpca_core::{AgentRuntime, MpcaConfig};
//! use std::path::PathBuf;
//!
//! // Create configuration
//! let config = MpcaConfig::new(PathBuf::from("/path/to/repo"));
//!
//! // Create runtime
//! let runtime = AgentRuntime::new(config)?;
//!
//! // Initialize repository
//! runtime.init_project()?;
//! ```

pub mod config;
pub mod error;
pub mod runtime;
pub mod state;
pub mod tools;
pub mod workflows;

// Re-export core types for convenience
pub use config::{
    AgentMode, GitConfig, MpcaConfig, ReviewConfig, ToolSet, WorkflowModes, WorkflowTools,
};
pub use error::{MPCAError, Result};
pub use runtime::AgentRuntime;
pub use state::{Phase, RuntimeState};
pub use tools::ToolRegistry;
