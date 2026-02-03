//! Prompt manager crate for MPCA.
//!
//! This crate provides template loading and rendering capabilities using minijinja.
//! It manages system prompts and custom templates for different workflow phases.
//!
//! # Examples
//!
//! ```no_run
//! use mpca_pm::{PromptManager, PromptEngine, PromptContext};
//! use std::path::PathBuf;
//!
//! let template_dir = PathBuf::from("./templates");
//! let manager = PromptManager::new(template_dir)?;
//!
//! let context = PromptContext::new(PathBuf::from("/my/repo"))
//!     .with_feature("add-caching");
//!
//! let prompt = manager.render("plan", &context)?;
//! println!("Generated prompt: {}", prompt);
//! # Ok::<(), mpca_pm::PromptError>(())
//! ```

pub mod context;
pub mod engine;
pub mod error;
pub mod manager;

// Re-export public types for convenience
pub use context::PromptContext;
pub use engine::PromptEngine;
pub use error::{PromptError, Result};
pub use manager::PromptManager;
