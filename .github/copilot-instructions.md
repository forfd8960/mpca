# MPCA - Mine Personal Coding Agent

MPCA is a Rust workspace that wraps the Claude Agent SDK to provide automated feature development workflows (`init`, `plan`, `run`). This project enables users to easily add new features to repositories through structured, AI-assisted development.

## Project Structure

This is a Rust workspace with three crates:
- `crates/mpca-core`: Core execution engine (Claude SDK integration, workflow orchestration, tool adapters)
- `crates/mpca-pm`: Prompt manager (template loading, rendering via minijinja)
- `apps/mpca-cli`: Command-line interface (clap for parsing, ratatui for TUI)

See `specs/design.md` for architecture diagrams, data structures, and detailed design.

## Development Guidelines

### Rust Standards
- **Edition**: Always use Rust 2024 edition. Pin in `rust-toolchain.toml`.
- **Quality Gates**: Run `cargo build`, `cargo test`, `cargo +nightly fmt`, and `cargo clippy -- -D warnings` before completing tasks.
- **Linting**: Use `cargo clippy -- -D warnings -W clippy::pedantic` for strict checks. Justify any allowed lints.
- **Security**: Run `cargo audit` regularly. Use `cargo-deny` for license enforcement.

### Error Handling (Critical for MPCA)
- **Never** use `unwrap()` or `expect()` in production code.
- Use `thiserror` for library errors (see `MPCAError` enum in design.md).
- Use `anyhow` for application-level errors with `.context()` for rich error messages.
- All fallible functions return `Result<T>`. Never use `Option` to represent errors.
- Define domain errors in `crates/mpca-core/src/error.rs` with clear variants for all failure modes.

### Async & Concurrency
- **Runtime**: Tokio with explicit features: `tokio = { version = "1.47", features = ["rt-multi-thread", "macros"] }`.
- **Patterns**: Prefer message passing (channels) over shared state. Use `tokio::sync::mpsc` or `flume`.
- **Actor Model**: Organize subsystems as actors with owned state and channel-based communication.
- **Non-blocking**: Use `tokio::task::spawn_blocking` for CPU-intensive or blocking operations.
- **Task Management**: Always handle spawned task panics. Use `tokio::task::JoinSet` for managing multiple tasks.

### Type Design
- Use `typed-builder` for structs with >5 fields (e.g., `MpcaConfig`, `RuntimeState`).
- Implement `Debug` for all types. Use `#[non_exhaustive]` for library types.
- Encode invariants in types (e.g., `Phase` enum for workflow states).
- Avoid `Option<T>` when `T` has a default (use empty Vec/HashMap instead).

### Testing
- **Unit tests**: Same file with `#[cfg(test)] mod tests`. Use `test_should_*` naming.
- **Integration tests**: In `tests/` directory for end-to-end workflows.
- **Parameterized tests**: Use `rstest` for data-driven tests.
- **Mocking**: Use `mockall` for adapters (`FsAdapter`, `GitAdapter`, `ShellAdapter`).
- **Coverage**: Focus on critical paths (workflow orchestration, error handling, state transitions).

### Logging
- Use `tracing` for structured logging. Never use `println!` or `dbg!` in production.
- Add `#[instrument(skip(large_param))]` on async functions for context.
- Levels: `error!` for failures, `warn!` for issues, `info!` for workflow steps, `debug!/trace!` for diagnostics.

### Dependencies
- **Workspace**: Define shared deps in root `Cargo.toml` under `[workspace.dependencies]`.
- **Core deps**: `tokio`, `anyhow`, `thiserror`, `serde`, `claude-agent-sdk-rs = "0.6"`.
- **PM deps**: `minijinja = "2.15"`, `serde`.
- **CLI deps**: `clap = "4.5"`, `ratatui = "0.30"`.
- Minimize dependencies. Prefer pure Rust over FFI bindings.

### Security
- Never log or expose sensitive data (API keys, tokens).
- Use `secrecy` crate for handling secrets.
- Use `dotenvy` for test environment variables. Never hardcode credentials.
- Validate all external input (user args, file content, API responses).

### Code Style
- Imports order: std → deps → local modules.
- Naming: `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
- Keep functions <150 lines. Extract complex logic into well-named helpers.
- Never use `todo!()`. Complete all code paths before submitting.
- Use trailing commas for cleaner diffs.

## MPCA-Specific Patterns

### Workflow Implementation
Each workflow (init, plan, execute, verify, review) should:
1. Load context and configuration
2. Render appropriate system prompt from `crates/mpca-pm/templates/*.j2`
3. Initialize Claude agent with correct mode and toolset (see design.md section 9)
4. Execute workflow steps with state tracking
5. Update `state.toml` after each step
6. Handle errors gracefully with resume capability

### Agent Modes & Tools
- **init**: Standard mode, minimal tools (fs write, git check)
- **plan**: Claude Code preset, standard tools (fs read, search)
- **execute**: Claude Code preset, full tools (fs, git, shell, test runner)
- **review**: Claude Code preset, standard tools (fs read, git diff)
- **verify**: Standard mode, standard tools (shell, test runner)

See `specs/design.md` section 9 for toolset definitions and configuration overrides.

### State Management
- All state in `specs/state.toml`: phase, step, turns, cost, timestamps
- Support resume from interruption (read last state, continue from checkpoint)
- Track costs and turns for transparency in PR descriptions

### Error Context
When errors occur:
- Capture in `MPCAError` with specific variant (e.g., `FeatureNotFound`, `WorktreeExists`)
- Add context with file paths, slugs, or relevant data
- Log with `error!` or `warn!` as appropriate
- Update state to indicate failure/blocker

### Template Rendering
Prompts in `crates/mpca-pm/templates/` use minijinja:
- All context variables are provided by engine (no inline defaults)
- Variables: `repo_root`, `feature_slug`, `specs_dir`, `worktree_dir`, `branch`, `resume`, etc.
- Render system prompts for agent initialization
- Keep templates declarative; engine resolves all conditionals

## Documentation Requirements
- Public APIs: Doc comments with examples, error/panic sections
- Modules: Use `//!` for purpose and usage patterns
- Examples: At least one example per public function (auto-tested)
- Architecture: Keep `specs/design.md` updated with structural changes

## Before Submitting
1. Run `cargo build` (all crates)
2. Run `cargo test` (all tests pass)
3. Run `cargo +nightly fmt` (formatting)
4. Run `cargo clippy -- -D warnings` (no warnings)
5. Update relevant docs in `specs/` if design changed
6. Verify error handling paths are covered

## References
- Design Document: `specs/design.md`
- Architecture Diagram: `specs/mpca_arch.png`
- Templates: `crates/mpca-pm/templates/*.j2`
- Error Types: See `MPCAError` enum in design.md section 5.1
