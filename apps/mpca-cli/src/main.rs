//! MPCA CLI - Mine Personal Coding Agent
//!
//! Command-line interface for MPCA, providing automated feature development
//! workflows through Claude Agent SDK integration.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mpca_core::{AgentRuntime, MpcaConfig};
use std::path::{Path, PathBuf};
use tracing::{error, info};

mod tui;

/// MPCA - Mine Personal Coding Agent
///
/// Automated feature development workflows using Claude Agent SDK.
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

/// Available MPCA commands
#[derive(Subcommand)]
enum Commands {
    /// Initialize repository for MPCA use
    ///
    /// Creates .mpca/ and .trees/ directories, generates default configuration,
    /// and updates CLAUDE.md with MPCA documentation.
    Init,

    /// Plan a new feature
    ///
    /// Interactively plan a feature by chatting with Claude to generate specs,
    /// design documents, and implementation plans.
    Plan {
        /// Feature slug (e.g., "add-caching")
        feature_name: String,

        /// Enable interactive TUI mode for planning
        #[arg(short, long)]
        interactive: bool,
    },

    /// Execute a planned feature
    ///
    /// Implements a previously planned feature by executing the implementation
    /// workflow in a dedicated git worktree.
    Run {
        /// Feature slug to execute
        feature_name: String,
    },

    /// Review feature changes before PR
    ///
    /// Review implemented changes, generate PR description, and prepare for
    /// merge back to main branch.
    Review {
        /// Feature slug to review
        feature_name: String,
    },

    /// Enter interactive chat mode
    ///
    /// Chat with Claude in the context of the current repository without
    /// committing to a specific feature workflow.
    Chat,

    /// Resume interrupted workflow
    ///
    /// Resume a previously interrupted workflow from its last checkpoint.
    Resume {
        /// Feature slug to resume
        feature_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing subscriber
    init_tracing(cli.verbose);

    // Execute command
    if let Err(e) = run_command(cli.command).await {
        // Log with tracing
        error!("Command failed: {:#}", e);
        // Also print to stderr for CLI users
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }

    Ok(())
}

/// Initialize tracing subscriber for structured logging
fn init_tracing(verbose: bool) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = if verbose {
        EnvFilter::new("mpca=debug,mpca_core=debug,mpca_pm=debug")
    } else {
        EnvFilter::new("mpca=info,mpca_core=info,mpca_pm=info")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .init();
}

/// Execute the specified command
async fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Init => {
            info!("Initializing repository for MPCA...");
            run_init().await
        }
        Commands::Plan {
            feature_name,
            interactive,
        } => {
            info!("Planning feature: {}", feature_name);
            run_plan(&feature_name, interactive).await
        }
        Commands::Run { feature_name } => {
            info!("Executing feature: {}", feature_name);
            run_execute(&feature_name).await
        }
        Commands::Review { feature_name } => {
            info!("Reviewing feature: {}", feature_name);
            run_review(&feature_name).await
        }
        Commands::Chat => {
            info!("Entering chat mode...");
            run_chat().await
        }
        Commands::Resume { feature_name } => {
            info!("Resuming feature: {}", feature_name);
            run_resume(&feature_name).await
        }
    }
}

/// Run the init command
async fn run_init() -> Result<()> {
    // Find repository root (current directory or parent with .git)
    let repo_root = find_repo_root()
        .context("Failed to find repository root - are you in a git repository?")?;

    info!("Repository root: {}", repo_root.display());

    // Create configuration
    let config = MpcaConfig::new(repo_root.clone());

    // Create runtime
    let runtime = AgentRuntime::new(config).context("Failed to create agent runtime")?;

    // Execute init workflow
    runtime
        .init_project()
        .context("Failed to initialize repository")?;

    println!("✔ Detected repository root");
    println!("✔ Created .mpca/ and .trees/ directories");
    println!("✔ Generated default configuration");
    println!("✔ Updated .gitignore to exclude .trees/");
    println!("✔ Updated CLAUDE.md with MPCA documentation");
    println!("\nRepository initialized for MPCA!");
    println!("\nNext steps:");
    println!("  mpca plan <feature-name>    Plan a new feature");
    println!("  mpca run <feature-name>     Execute a planned feature");
    println!("  mpca chat                   Chat with Claude");

    Ok(())
}

/// Run the plan command
async fn run_plan(feature_name: &str, interactive: bool) -> Result<()> {
    // Find repository root
    let repo_root = find_repo_root()
        .context("Failed to find repository root - are you in a git repository?")?;

    // Load configuration
    let config = load_config(&repo_root).context("Failed to load MPCA configuration")?;

    // Create runtime
    let runtime = AgentRuntime::new(config).context("Failed to create agent runtime")?;

    if interactive {
        // Run interactive TUI mode
        info!("Starting interactive planning TUI...");
        tui::run_planning_tui(feature_name, &runtime)
            .await
            .context("Interactive planning failed")?;
    } else {
        // Run non-interactive planning (stub for now)
        info!("Planning feature: {}", feature_name);
        runtime
            .plan_feature(feature_name)
            .context("Feature planning failed")?;
        println!("✔ Feature planned: {}", feature_name);
        println!("\nNext steps:");
        println!("  mpca run {}    Execute this feature", feature_name);
    }

    Ok(())
}

/// Run the execute command
async fn run_execute(feature_name: &str) -> Result<()> {
    // Find repository root
    let repo_root = find_repo_root()
        .context("Failed to find repository root - are you in a git repository?")?;

    // Load configuration
    let config = load_config(&repo_root).context("Failed to load MPCA configuration")?;

    // Create runtime
    let runtime = AgentRuntime::new(config).context("Failed to create agent runtime")?;

    // Execute feature (stub for now)
    runtime
        .run_feature(feature_name)
        .context("Feature execution failed")?;

    println!("✔ Feature executed: {}", feature_name);
    println!("\nNext steps:");
    println!("  mpca review {}    Review changes", feature_name);

    Ok(())
}

/// Run the review command
async fn run_review(feature_name: &str) -> Result<()> {
    // Find repository root
    let repo_root = find_repo_root()
        .context("Failed to find repository root - are you in a git repository?")?;

    // Load configuration
    let _config = load_config(&repo_root).context("Failed to load MPCA configuration")?;

    // Review feature (stub for now)
    println!("✔ Reviewing feature: {}", feature_name);
    println!("\nFeature review complete.");
    println!("\nNext steps:");
    println!("  cd .trees/{}", feature_name);
    println!("  git push -u origin feature/{}", feature_name);

    Ok(())
}

/// Run the chat command
async fn run_chat() -> Result<()> {
    // Find repository root
    let repo_root = find_repo_root()
        .context("Failed to find repository root - are you in a git repository?")?;

    // Load configuration
    let _config = load_config(&repo_root).context("Failed to load MPCA configuration")?;

    // Chat mode (stub for now)
    println!("Chat mode is not yet implemented.");
    println!("This will enable interactive conversation with Claude.");

    Ok(())
}

/// Run the resume command
async fn run_resume(feature_name: &str) -> Result<()> {
    // Find repository root
    let repo_root = find_repo_root()
        .context("Failed to find repository root - are you in a git repository?")?;

    // Load configuration
    let _config = load_config(&repo_root).context("Failed to load MPCA configuration")?;

    // Resume workflow (stub for now)
    println!("✔ Resuming feature: {}", feature_name);
    println!("\nResume functionality is not yet implemented.");

    Ok(())
}

/// Find the repository root by searching for .git directory
fn find_repo_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Search up the directory tree for .git
    let mut path = current_dir.as_path();
    loop {
        if path.join(".git").exists() {
            return Ok(path.to_path_buf());
        }

        match path.parent() {
            Some(parent) => path = parent,
            None => {
                anyhow::bail!("Not a git repository (or any parent up to mount point)")
            }
        }
    }
}

/// Load MPCA configuration from .mpca/config.toml
fn load_config(repo_root: &Path) -> Result<MpcaConfig> {
    // Check if .mpca directory exists
    let mpca_dir = repo_root.join(".mpca");
    if !mpca_dir.exists() {
        anyhow::bail!(
            "MPCA not initialized. Run 'mpca init' first.\n\
             Expected directory: {}",
            mpca_dir.display()
        );
    }

    // For now, just return default config
    // TODO: Load from config.toml file in Stage 5
    Ok(MpcaConfig::new(repo_root.to_path_buf()))
}
