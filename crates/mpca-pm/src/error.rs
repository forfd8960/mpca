//! Error types for the prompt manager crate.

use std::path::PathBuf;

/// Errors that can occur in the prompt manager.
#[derive(thiserror::Error, Debug)]
pub enum PromptError {
    /// Template file was not found in the templates directory.
    #[error("template not found: {0}")]
    TemplateNotFound(String),

    /// Error occurred while rendering a template.
    #[error("template render error: {0}")]
    TemplateRenderError(String),

    /// Invalid or missing context data required for template rendering.
    #[error("invalid template context: {0}")]
    InvalidTemplateContext(String),

    /// Failed to load or read template from filesystem.
    #[error("template load error: {path}")]
    TemplateLoadError {
        /// Path to the template that failed to load.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Template directory does not exist or is not accessible.
    #[error("template directory not found: {0}")]
    TemplateDirectoryNotFound(PathBuf),

    /// Template directory listing failed.
    #[error("failed to list templates in {path}")]
    TemplateListError {
        /// Path to the template directory.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Serialization error when preparing context data.
    #[error("context serialization error: {0}")]
    ContextSerializationError(String),
}

/// Result type alias for prompt manager operations.
pub type Result<T> = std::result::Result<T, PromptError>;
