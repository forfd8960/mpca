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
//!
//! # Example
//!
//! ```rust,ignore
//! use mpca_core::{MpcaConfig, RuntimeState, Phase};
//! use std::path::PathBuf;
//!
//! // Create configuration
//! let config = MpcaConfig::new(PathBuf::from("/path/to/repo"));
//!
//! // Create runtime state
//! let mut state = RuntimeState::for_feature("my-feature");
//! assert_eq!(state.phase, Phase::Plan);
//!
//! // Advance through workflow
//! state.advance_phase();
//! assert_eq!(state.phase, Phase::Run);
//! ```

pub mod config;
pub mod error;
pub mod state;
pub mod tools;

// Re-export core types for convenience
pub use config::{
    AgentMode, GitConfig, MpcaConfig, ReviewConfig, ToolSet, WorkflowModes, WorkflowTools,
};
pub use error::{MPCAError, Result};
pub use state::{Phase, RuntimeState};
pub use tools::ToolRegistry;
