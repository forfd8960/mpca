# MPCA (Mine Personal Coding Agent) Design Document

## 1. Architecture Overview

MPCA is a Rust workspace that wraps the Claude Agent SDK to provide three user-facing workflows: `mpca init`, `mpca plan`, and `mpca run`. The system is split into three crates for single responsibility: `mpca-cli` (UI/UX), `mpca-core` (execution engine), and `mpca-pm` (prompt manager).

### High-Level Architecture (ASCII)

```
+---------------------------------------------------------------+
|                        User (Terminal)                        |
+-------------------------------+-------------------------------+
                                |
                                v
+---------------------------------------------------------------+
|                         mpca-cli                              |
|  - clap: command parsing          - ratatui: TUI (plan)        |
|  Commands: init | plan | run                                    |
+-------------------------------+-------------------------------+
                                |
            +-------------------+-------------------+
            |                                       |
            v                                       v
+-----------------------+                   +-------------------+
|       mpca-core       | <---------------> |      mpca-pm      |
|  Execution Engine     |   prompt render   | Prompt Manager    |
|  - workflow state     |                   | - templates       |
|  - agent runtime      |                   | - context loading |
|  - tool adapters      |                   +-------------------+
+-----------------------+
            |
            v
+---------------------------------------------------------------+
|                  claude-agent-sdk-rs (0.6.x)                  |
+---------------------------------------------------------------+
            |
            v
+---------------------------------------------------------------+
|        OS / Repo Services (FS, Git, Shell, Network)           |
+---------------------------------------------------------------+
```

### Key Processes (ASCII)

#### `mpca init`

```
[User] -> [mpca init]
            |
            v
      [Scan repo] -> [Create .mpca/ + .trees/]
            |
            v
      [Generate specs skeletons] -> [Update CLAUDE.md links]
```

#### `mpca plan <feature-slug>`

```
[User] -> [mpca plan feature-slug]
            |
            v
      [ratatui TUI loop] <-> [mpca-core chat] <-> [Claude SDK]
            |
            v
      [Save specs] -> [Create git worktree (.trees/feature)]
```

#### `mpca run <feature-slug>`

```
[User] -> [mpca run feature-slug]
            |
            v
      [Load specs] -> [Observer] -> [Implement] -> [Verify]
            |
            v
      [Commit changes] -> [Finalize report]
```

## 2. Directory Structure Managed by `mpca-cli`

The CLI creates and manages the following project-local structure (relative to repo root):

```
.
├─ .mpca/
│  ├─ config.toml
│  ├─ specs/
│  │  └─ <feature-slug>/
│  │     ├─ specs/
│  │     │  ├─ state.toml
│  │     │  ├─ design.md
│  │     │  ├─ plan.md
│  │     │  └─ verify.md
│  │     └─ docs/
│  │        ├─ impl_details.md
│  │        └─ review.md
├─ .trees/
│  └─ <feature-slug>/
├─ CLAUDE.md
└─ .gitignore  (contains .trees)
```

### `.mpca/config.toml` (Core Options)

- Override default agent behavior (model, permission mode, etc.)
- Specify additional prompt template directories
- Configure Git behavior (auto-commit, branch naming)
- Configure code review options

## 3. Code Directory Structure

The MPCA workspace is organized as follows:

```sh
mpca/
├─ Cargo.toml                    # Workspace manifest
├─ README.md
├─ LICENSE.md
├─ CHANGELOG.md
├─ Makefile
├─ .gitignore
├─ crates/
│  ├─ mpca-core/
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     ├─ lib.rs             # Public API & re-exports
│  │     ├─ error.rs           # MPCAError enum
│  │     ├─ config.rs          # MpcaConfig, GitConfig, ReviewConfig
│  │     ├─ state.rs           # RuntimeState, Phase
│  │     ├─ runtime.rs         # AgentRuntime implementation
│  │     ├─ tools/
│  │     │  ├─ mod.rs          # ToolRegistry
│  │     │  ├─ fs.rs           # FsAdapter trait & impl
│  │     │  ├─ git.rs          # GitAdapter trait & impl
│  │     │  └─ shell.rs        # ShellAdapter trait & impl
│  │     └─ workflows/
│  │        ├─ mod.rs
│  │        ├─ init.rs         # init_project workflow
│  │        ├─ plan.rs         # plan_feature workflow
│  │        ├─ run.rs          # run_feature workflow
│  │        └─ verify.rs       # verification workflow
│  └─ mpca-pm/
│     ├─ Cargo.toml
│     ├─ src/
│     │  ├─ lib.rs             # Public API & re-exports
│     │  ├─ error.rs           # Prompt-specific errors
│     │  ├─ manager.rs         # PromptManager implementation
│     │  ├─ context.rs         # PromptContext
│     │  └─ engine.rs          # PromptEngine trait
│     └─ templates/
│        ├─ init.j2            # Init workflow system prompt
│        ├─ plan.j2            # Plan workflow system prompt
│        ├─ execute.j2         # Execute workflow system prompt
│        ├─ review.j2          # Review workflow system prompt
│        └─ verification.j2    # Verification workflow system prompt
├─ apps/
│  └─ mpca-cli/
│     ├─ Cargo.toml
│     └─ src/
│        ├─ main.rs            # CLI entry point
│        ├─ cli.rs             # clap command definitions
│        ├─ tui/
│        │  ├─ mod.rs
│        │  ├─ app.rs          # ratatui app state
│        │  ├─ ui.rs           # UI rendering
│        │  └─ events.rs       # Event handling
│        └─ commands/
│           ├─ mod.rs
│           ├─ init.rs         # mpca init command
│           ├─ plan.rs         # mpca plan command
│           └─ run.rs          # mpca run command
├─ specs/
│  ├─ design.md                # This document
│  ├─ instructions.md
│  └─ mpca_arch.png
├─ examples/
│  └─ README.md
└─ fixtures/
   └─ README.md
```

## 4. Example Output for `mpca init`

```
$ mpca init
Initialize current project for MPCA...
✔ detected repository root
✔ created .mpca/ and .trees/
✔ generated feature spec layout under .mpca/specs/<feature-slug>/
✔ updated CLAUDE.md with links to .mpca/specs
Done.
```

## 5. Crate Responsibilities & Public Interfaces

### 5.1 `mpca-core` (Core Execution Engine)

**Responsibility**: Orchestrates workflows, manages runtime state, calls the Claude Agent SDK, and invokes tool adapters (FS/Git/Shell). Designed to expose a minimal, task-oriented API.

**Public Interface (concise)**:
- `AgentRuntime::new(config)`
- `AgentRuntime::init_project()`
- `AgentRuntime::plan_feature(feature_slug)`
- `AgentRuntime::run_feature(feature_slug)`
- `AgentRuntime::chat(message)`

**Core public traits & data structures**:

```rust
pub trait Runtime {
      fn init_project(&self) -> Result<()>;
      fn plan_feature(&self, feature_slug: &str) -> Result<()>;
      fn run_feature(&self, feature_slug: &str) -> Result<()>;
      fn chat(&self, message: &str) -> Result<String>;
}

pub struct AgentRuntime {
      pub config: MpcaConfig,
      pub pm: PromptManager,
      pub tools: ToolRegistry,
      pub state: RuntimeState,
}

pub struct MpcaConfig {
      pub repo_root: PathBuf,
      pub trees_dir: PathBuf,
      pub specs_dir: PathBuf,
      pub claude_md: PathBuf,
      pub config_file: PathBuf,
      pub prompt_dirs: Vec<PathBuf>,
      pub git: GitConfig,
      pub review: ReviewConfig,
}

pub struct RuntimeState {
      pub feature_slug: Option<String>,
      pub phase: Phase,
      pub turns: u32,
      pub cost_usd: f64,
}

pub enum Phase {
      Init,
      Plan,
      Run,
      Verify,
}

pub struct ToolRegistry {
      pub fs: Box<dyn FsAdapter>,
      pub git: Box<dyn GitAdapter>,
      pub shell: Box<dyn ShellAdapter>,
}

pub struct GitConfig {
      pub auto_commit: bool,
      pub branch_naming: String,
}

pub struct ReviewConfig {
      pub enabled: bool,
      pub reviewers: Vec<String>,
}

pub struct ApiConfig {
      /// Base URL for Claude API endpoint (optional)
      /// When None, uses SDK default (https://api.anthropic.com)
      pub base_url: Option<String>,
}

pub trait FsAdapter { /* read/write/list */ }
pub trait GitAdapter { /* worktree/commit/status */ }
pub trait ShellAdapter { /* run/stream */ }

/// Comprehensive error types for MPCA
#[derive(thiserror::Error, Debug)]
pub enum MPCAError {
      // Initialization errors
      #[error("not a git repository: {0}")]
      NotGitRepository(PathBuf),
      #[error("repo already initialized by MPCA")]
      AlreadyInitialized,
      #[error("repo not initialized by MPCA - run `mpca init` first")]
      NotInitialized,
      #[error("permission denied: {0}")]
      PermissionDenied(String),

      // Feature errors
      #[error("feature not found: {0}")]
      FeatureNotFound(String),
      #[error("feature already exists: {0}")]
      FeatureAlreadyExists(String),
      #[error("invalid feature slug: {0} (must be lowercase alphanumeric with hyphens)")]
      InvalidFeatureSlug(String),

      // State errors
      #[error("corrupted state file: {0}")]
      CorruptedState(PathBuf),
      #[error("invalid state transition from {0} to {1}")]
      InvalidStateTransition(String, String),
      #[error("state file missing: {0}")]
      StateMissing(PathBuf),

      // Git errors
      #[error("worktree already exists: {0}")]
      WorktreeExists(PathBuf),
      #[error("branch already exists: {0}")]
      BranchExists(String),
      #[error("uncommitted changes in worktree: {0}")]
      UncommittedChanges(PathBuf),
      #[error("git command failed: {0}")]
      GitCommandFailed(String),
      #[error("worktree not found: {0}")]
      WorktreeNotFound(PathBuf),

      // File system errors
      #[error("path not found: {0}")]
      PathNotFound(PathBuf),
      #[error("invalid path: {0}")]
      InvalidPath(PathBuf),
      #[error("file read error: {0}")]
      FileReadError(String),
      #[error("file write error: {0}")]
      FileWriteError(String),

      // Config errors
      #[error("invalid config: {0}")]
      InvalidConfig(String),
      #[error("config parse error: {0}")]
      ConfigParseError(String),
      #[error("missing required config field: {0}")]
      MissingConfigField(String),
      #[error("config file not found: {0}")]
      ConfigNotFound(PathBuf),

      // Prompt/template errors
      #[error("template not found: {0}")]
      TemplateNotFound(String),
      #[error("template render error: {0}")]
      TemplateRenderError(String),
      #[error("invalid template context: {0}")]
      InvalidTemplateContext(String),

      // Agent/SDK errors
      #[error("claude agent error: {0}")]
      AgentError(String),
      #[error("API authentication failed")]
      AuthenticationFailed,
      #[error("API rate limit exceeded")]
      RateLimitExceeded,
      #[error("agent timeout after {0}s")]
      AgentTimeout(u64),

      // Plan errors
      #[error("invalid plan format: {0}")]
      InvalidPlanFormat(String),
      #[error("plan validation failed: {0}")]
      PlanValidationFailed(String),
      #[error("missing plan section: {0}")]
      MissingPlanSection(String),
      #[error("plan not found for feature: {0}")]
      PlanNotFound(String),

      // Verification errors
      #[error("verification failed: {0}")]
      VerificationFailed(String),
      #[error("tests failed: {0}")]
      TestsFailed(String),
      #[error("verification spec missing for feature: {0}")]
      VerificationSpecMissing(String),
      #[error("verification timeout after {0}s")]
      VerificationTimeout(u64),

      // Tool/adapter errors
      #[error("shell command failed: {0}")]
      ShellCommandFailed(String),
      #[error("tool execution error: {0}")]
      ToolExecutionError(String),

      // IO and system errors
      #[error("io error: {0}")]
      Io(#[from] std::io::Error),

      // Generic fallback
      #[error("unexpected error: {0}")]
      Other(String),
}

pub type Result<T> = std::result::Result<T, MPCAError>;
```

### 5.2 `mpca-pm` (Prompt Manager)

**Responsibility**: Loads, renders, and manages prompt templates. Provides an ergonomic API over `minijinja` for system and task prompts.

**Public Interface (concise)**:
- `PromptManager::new(template_dir)`
- `PromptManager::render(template_name, context)`
- `PromptManager::get_system_prompt(role)`
- `PromptManager::list_templates()`

**Core public traits & data structures**:

```rust
pub trait PromptEngine {
      fn render<T: Serialize>(&self, template: &str, ctx: &T) -> Result<String>;
      fn get_system_prompt(&self, role: &str) -> Result<String>;
      fn list_templates(&self) -> Result<Vec<String>>;
}

pub struct PromptManager {
      pub templates_dir: PathBuf,
      pub env: minijinja::Environment<'static>,
}

pub struct PromptContext {
      pub repo_root: PathBuf,
      pub feature_slug: Option<String>,
      pub spec_paths: Vec<PathBuf>,
      pub resume: bool,
}
```

### 5.3 `mpca-cli` (Command Line Interface)

**Responsibility**: Owns CLI UX, parses commands (`clap`), runs TUI flows (`ratatui`), and delegates logic to `mpca-core`.

**Public Surface**:
- `mpca init`
- `mpca plan <feature-slug>`
- `mpca run <feature-slug>`

## 6. Design Principles

- **Single Responsibility**: Each crate owns one domain (UI, runtime, prompts).
- **SOLID**: Use small traits for adapters (FS/Git/Shell) and inject them into `mpca-core`.
- **No duplication**: Prompt rendering is centralized in `mpca-pm`; CLI is a thin wrapper over `mpca-core`.
- **Modern Rust**: Use `edition = 2024`, `tokio` async/await, and structured error types.
- **Resumable runs**: If `mpca run` is interrupted, it resumes from the last saved state.

## 7. State & Results Recording

- Task execution results (turns, cost) are recorded in `state.toml` and referenced in the final PR link.
- Each feature has `specs/state.toml` as the authoritative state file for progress and step tracking.

## 8. Prompt Planning & Storage

- Pre-plan prompts for all scenarios and store them in `crates/mpca-pm/templates` for review.

## 9. Agent Mode & Tool Configuration

### Scenario-Based Configuration

Different workflows require different agent capabilities and tool access. Following convention-over-configuration, the engine provides sensible defaults while allowing overrides via `.mpca/config.toml`.

| Workflow | Agent Mode | Tool Set | Rationale |
|----------|------------|----------|-----------|
| **init** | Standard | fs (write), git (check) | Simple setup task; no code analysis needed |
| **plan** | Claude Code | fs (read), semantic_search, grep | Interactive planning benefits from code context understanding |
| **execute** | Claude Code | fs (full), git (full), shell, test_runner | Autonomous coding requires full tool access and code understanding |
| **review** | Claude Code | fs (read), git (diff), linters | Code review needs context and diff analysis |
| **verify** | Standard | shell, fs (read), test_runner | Test execution doesn't require code generation |

### Default Tool Sets

Defined in `mpca-core/src/tools/mod.rs`:

```rust
pub enum ToolSet {
    Minimal,    // fs (read), git (status)
    Standard,   // fs (read/write), git (status/commit), shell (limited)
    Full,       // fs (full), git (full), shell (full), test_runner, search
}

impl Default for WorkflowTools {
    fn default() -> Self {
        Self {
            init: ToolSet::Minimal,
            plan: ToolSet::Standard,
            execute: ToolSet::Full,
            review: ToolSet::Standard,
            verify: ToolSet::Standard,
        }
    }
}
```

### Agent Mode Configuration

Defined in `mpca-core/src/config.rs`:

```rust
pub struct AgentMode {
    pub use_code_preset: bool,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for WorkflowModes {
    fn default() -> Self {
        Self {
            init: AgentMode { use_code_preset: false, model: "claude-3-5-sonnet-20241022".into(), temperature: 0.0, max_tokens: 4096 },
            plan: AgentMode { use_code_preset: true, model: "claude-3-5-sonnet-20241022".into(), temperature: 0.3, max_tokens: 8192 },
            execute: AgentMode { use_code_preset: true, model: "claude-3-5-sonnet-20241022".into(), temperature: 0.0, max_tokens: 8192 },
            review: AgentMode { use_code_preset: true, model: "claude-3-5-sonnet-20241022".into(), temperature: 0.0, max_tokens: 8192 },
            verify: AgentMode { use_code_preset: false, model: "claude-3-5-sonnet-20241022".into(), temperature: 0.0, max_tokens: 4096 },
        }
    }
}
```

### Configuration Override

Users can override defaults in `.mpca/config.toml`:

```toml
# API configuration for Claude SDK
[api]
# Optional: Override default Claude API endpoint
# Useful for custom deployments, proxies, or testing
# base_url = "https://api.anthropic.com"

# Agent configuration per workflow
[agent_modes.init]
use_code_preset = false
model = "claude-3-5-sonnet-20241022"

[agent_modes.plan]
use_code_preset = true
temperature = 0.3

[agent.execute]
use_code_preset = true
tools = "full"  # or "standard", "minimal"

[agent.review]
use_code_preset = true

[agent.verify]
use_code_preset = false
```

### Implementation Location

- **Engine (`mpca-core`)**: Provides default configurations, workflow-to-toolset mapping, agent initialization
- **Config (`.mpca/config.toml`)**: User overrides for specific needs (e.g., use cheaper model for verify, different temperature for planning)
- **Runtime**: Merges defaults + config at startup, validates tool availability

### Design Rationale

1. **Convention over Configuration**: Sensible defaults mean zero-config works well
2. **Escape Hatches**: Power users can tune per-workflow settings
3. **Safety**: Execute gets full tools, but others are restricted by default
4. **Cost Optimization**: Verify and init don't need expensive code context
5. **Flexibility**: Config file allows experimentation without code changes

## 10. Development Plan

### Stage 1: Workspace & Types
- Define core data types (config, plan, spec models).
- Establish adapter traits for FS/Git/Shell.

### Stage 2: Prompt Manager (`mpca-pm`)
- Load templates from `specs/` or embedded assets.
- Render system and task prompts with `minijinja`.
- Add unit tests for template rendering.

### Stage 3: Core Runtime (`mpca-core`)
- Implement `init_project` workflow for `.mpca/` and `.trees/`.
- Integrate `claude-agent-sdk-rs` for chat and tool calls.
- Provide structured status/progress events for the CLI.

### Stage 4: CLI & TUI (`mpca-cli`)
- Implement command parsing for `init`, `plan`, `run`.
- Build ratatui TUI loop for `plan`.
- Stream runtime events for `run`.

### Stage 5: End-to-End Flow
- Connect plan outputs to run inputs.
- Git worktree automation and spec validation.
- Error recovery and resumable workflows.

### Stage 6: Verification & Polish
- Integration tests with fixture repos.
- Docs and examples in `specs/` and `examples/`.
