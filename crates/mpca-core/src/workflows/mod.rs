//! Workflow modules for MPCA.
//!
//! This module organizes the different workflow implementations:
//! - `init`: Initialize a repository for MPCA use
//! - `plan`: Plan a new feature (to be implemented)
//! - `run`: Execute a feature plan (to be implemented)
//! - `verify`: Verify implementation (to be implemented)

pub mod init;

// Re-export workflow functions
pub use init::init_project;
