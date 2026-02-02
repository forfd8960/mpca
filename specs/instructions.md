# Instruction

## Init Repo

This is a `Mine Personal Coding Agent` Project. Its main function is to wrap the Claude Agent SDK, allowing users to easily add new features around a repository. Please convert this Rust project into a workspace containing `crates/mpca-core` (core execution engine), `crates/mpca-pm` (prompt manager), and `apps/mpca-cli` (command line interface). The generated output is a `mpca` CLI. All deps should be placed under the workspace, and each crate should be referenced using `crate-name = { workspace = true }`. The CLI is built using `clap, ratatai`. The prompt manager is built using `minijinja`. The core execution engine is built using `tokio/claude-agent-sdk-rs 0.6`. All deps must be the latest versions.

Please do not write any code yet; simply generate the skeletons for each crate.
