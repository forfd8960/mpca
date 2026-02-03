//! Core prompt engine trait definition.

use crate::error::Result;
use serde::Serialize;

/// Trait for rendering templates with dynamic context.
///
/// Implementations of this trait handle loading, caching, and rendering
/// of templates using a template engine like minijinja.
///
/// # Examples
///
/// ```no_run
/// use mpca_pm::{PromptEngine, PromptContext, PromptManager};
/// use std::path::PathBuf;
///
/// fn render_example(engine: &PromptManager) -> Result<(), Box<dyn std::error::Error>> {
///     let context = PromptContext::new(PathBuf::from("/repo"));
///     let rendered = engine.render("init", &context)?;
///     println!("Rendered prompt: {}", rendered);
///     Ok(())
/// }
/// ```
pub trait PromptEngine {
    /// Renders a template with the provided context.
    ///
    /// # Arguments
    ///
    /// * `template` - Name of the template to render (without extension)
    /// * `ctx` - Context data to use for rendering
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template does not exist
    /// - The context cannot be serialized
    /// - The template contains syntax errors
    /// - Template rendering fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mpca_pm::{PromptEngine, PromptContext, PromptManager};
    /// # use std::path::PathBuf;
    /// # fn example(engine: &PromptManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let context = PromptContext::new(PathBuf::from("/repo"));
    /// let prompt = engine.render("plan", &context)?;
    /// # Ok(())
    /// # }
    /// ```
    fn render<T: Serialize>(&self, template: &str, ctx: &T) -> Result<String>;

    /// Gets a system prompt for a specific role.
    ///
    /// This is a convenience method that renders a template with minimal context,
    /// typically used for role-based system prompts.
    ///
    /// # Arguments
    ///
    /// * `role` - The role/template name (e.g., "init", "plan", "execute")
    ///
    /// # Errors
    ///
    /// Returns an error if the template for the role does not exist or rendering fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mpca_pm::{PromptEngine, PromptManager};
    /// # use std::path::PathBuf;
    /// # fn example(engine: &PromptManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let system_prompt = engine.get_system_prompt("execute")?;
    /// # Ok(())
    /// # }
    /// ```
    fn get_system_prompt(&self, role: &str) -> Result<String>;

    /// Lists all available templates.
    ///
    /// Returns a vector of template names (without extensions) that are available
    /// for rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if the template directory cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mpca_pm::{PromptEngine, PromptManager};
    /// # use std::path::PathBuf;
    /// # fn example(engine: &PromptManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let templates = engine.list_templates()?;
    /// for template in templates {
    ///     println!("Available template: {}", template);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn list_templates(&self) -> Result<Vec<String>>;
}
