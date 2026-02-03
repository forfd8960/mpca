//! Context structures for template rendering.

use serde::Serialize;
use std::path::PathBuf;

/// Context data provided to templates for rendering.
///
/// This structure contains all the dynamic information that templates
/// may need to generate prompts for different workflow phases.
///
/// # Examples
///
/// ```
/// use mpca_pm::PromptContext;
/// use std::path::PathBuf;
///
/// let context = PromptContext {
///     repo_root: PathBuf::from("/path/to/repo"),
///     feature_slug: Some("add-caching".to_string()),
///     spec_paths: vec![PathBuf::from(".mpca/specs/add-caching")],
///     resume: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct PromptContext {
    /// Absolute path to the repository root directory.
    pub repo_root: PathBuf,

    /// Current feature slug being worked on (if applicable).
    pub feature_slug: Option<String>,

    /// Paths to specification files or directories relevant to this context.
    pub spec_paths: Vec<PathBuf>,

    /// Whether this is a resumed workflow (true) or a fresh start (false).
    pub resume: bool,
}

impl Default for PromptContext {
    fn default() -> Self {
        Self {
            repo_root: PathBuf::new(),
            feature_slug: None,
            spec_paths: Vec::new(),
            resume: false,
        }
    }
}

impl PromptContext {
    /// Creates a new `PromptContext` with minimal required fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use mpca_pm::PromptContext;
    /// use std::path::PathBuf;
    ///
    /// let context = PromptContext::new(PathBuf::from("/repo"));
    /// assert_eq!(context.repo_root, PathBuf::from("/repo"));
    /// assert!(context.feature_slug.is_none());
    /// ```
    #[must_use]
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            ..Default::default()
        }
    }

    /// Sets the feature slug for this context.
    ///
    /// # Examples
    ///
    /// ```
    /// use mpca_pm::PromptContext;
    /// use std::path::PathBuf;
    ///
    /// let context = PromptContext::new(PathBuf::from("/repo"))
    ///     .with_feature("my-feature");
    /// assert_eq!(context.feature_slug, Some("my-feature".to_string()));
    /// ```
    #[must_use]
    pub fn with_feature(mut self, slug: impl Into<String>) -> Self {
        self.feature_slug = Some(slug.into());
        self
    }

    /// Sets the spec paths for this context.
    ///
    /// # Examples
    ///
    /// ```
    /// use mpca_pm::PromptContext;
    /// use std::path::PathBuf;
    ///
    /// let context = PromptContext::new(PathBuf::from("/repo"))
    ///     .with_spec_paths(vec![PathBuf::from("specs/design.md")]);
    /// assert_eq!(context.spec_paths.len(), 1);
    /// ```
    #[must_use]
    pub fn with_spec_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.spec_paths = paths;
        self
    }

    /// Sets the resume flag for this context.
    ///
    /// # Examples
    ///
    /// ```
    /// use mpca_pm::PromptContext;
    /// use std::path::PathBuf;
    ///
    /// let context = PromptContext::new(PathBuf::from("/repo"))
    ///     .with_resume(true);
    /// assert!(context.resume);
    /// ```
    #[must_use]
    pub fn with_resume(mut self, resume: bool) -> Self {
        self.resume = resume;
        self
    }
}
