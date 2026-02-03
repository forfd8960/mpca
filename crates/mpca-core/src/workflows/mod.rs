//! Workflow modules for MPCA.
//!
//! This module organizes the different workflow implementations:
//! - `init`: Initialize a repository for MPCA use
//! - `plan`: Plan a new feature
//! - `execute`: Execute a feature plan
//! - `verify`: Verify implementation (to be implemented)

pub mod execute;
pub mod init;
pub mod plan;

// Re-export workflow functions
pub use execute::execute_feature;
pub use init::init_project;
pub use plan::plan_feature;
