//! Runtime state management for MPCA workflows.
//!
//! This module defines the runtime state that tracks workflow progress,
//! including the current phase, turn count, and cost tracking.

use std::fmt;
use std::str::FromStr;

/// Runtime state for MPCA workflows.
///
/// Tracks the current execution state of a feature workflow, including
/// which phase it's in, how many agent turns have occurred, and the
/// cumulative cost. This state is persisted to `state.toml` to enable
/// resumable workflows.
#[derive(Debug, Clone)]
pub struct RuntimeState {
    /// Currently active feature slug (if any).
    pub feature_slug: Option<String>,

    /// Current workflow phase.
    pub phase: Phase,

    /// Number of agent turns executed so far.
    pub turns: u32,

    /// Cumulative cost in USD for agent API calls.
    pub cost_usd: f64,
}

impl RuntimeState {
    /// Creates a new runtime state with default values.
    ///
    /// # Returns
    ///
    /// A new `RuntimeState` with no active feature, phase set to `Init`,
    /// zero turns, and zero cost.
    pub fn new() -> Self {
        Self {
            feature_slug: None,
            phase: Phase::Init,
            turns: 0,
            cost_usd: 0.0,
        }
    }

    /// Creates a new runtime state for a specific feature.
    ///
    /// # Arguments
    ///
    /// * `feature_slug` - The feature slug identifier.
    ///
    /// # Returns
    ///
    /// A new `RuntimeState` with the specified feature, phase set to `Plan`,
    /// zero turns, and zero cost.
    pub fn for_feature(feature_slug: impl Into<String>) -> Self {
        Self {
            feature_slug: Some(feature_slug.into()),
            phase: Phase::Plan,
            turns: 0,
            cost_usd: 0.0,
        }
    }

    /// Advances to the next phase in the workflow.
    ///
    /// # Returns
    ///
    /// `true` if the phase was advanced, `false` if already in the final phase.
    pub fn advance_phase(&mut self) -> bool {
        match self.phase {
            Phase::Init => {
                self.phase = Phase::Plan;
                true
            }
            Phase::Plan => {
                self.phase = Phase::Run;
                true
            }
            Phase::Run => {
                self.phase = Phase::Verify;
                true
            }
            Phase::Verify => false,
        }
    }

    /// Increments the turn counter.
    pub fn increment_turn(&mut self) {
        self.turns += 1;
    }

    /// Adds to the cumulative cost.
    ///
    /// # Arguments
    ///
    /// * `cost` - The cost to add in USD.
    pub fn add_cost(&mut self, cost: f64) {
        self.cost_usd += cost;
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow phase enumeration.
///
/// Represents the different phases of an MPCA feature workflow.
/// Phases are sequential and non-reversible.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Initial setup phase (repository initialization).
    Init,

    /// Planning phase (interactive specification and design).
    Plan,

    /// Execution phase (automated implementation).
    Run,

    /// Verification phase (testing and validation).
    Verify,
}

impl Phase {
    /// Returns the string representation of the phase.
    ///
    /// This is used for serialization to `state.toml` and display purposes.
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Init => "init",
            Phase::Plan => "plan",
            Phase::Run => "run",
            Phase::Verify => "verify",
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Phase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(Phase::Init),
            "plan" => Ok(Phase::Plan),
            "run" => Ok(Phase::Run),
            "verify" => Ok(Phase::Verify),
            _ => Err(format!("invalid phase: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_default_state() {
        let state = RuntimeState::new();
        assert!(state.feature_slug.is_none());
        assert_eq!(state.phase, Phase::Init);
        assert_eq!(state.turns, 0);
        assert_eq!(state.cost_usd, 0.0);
    }

    #[test]
    fn test_should_create_state_for_feature() {
        let state = RuntimeState::for_feature("test-feature");
        assert_eq!(state.feature_slug, Some("test-feature".to_string()));
        assert_eq!(state.phase, Phase::Plan);
        assert_eq!(state.turns, 0);
        assert_eq!(state.cost_usd, 0.0);
    }

    #[test]
    fn test_should_advance_phases_sequentially() {
        let mut state = RuntimeState::new();
        assert_eq!(state.phase, Phase::Init);

        assert!(state.advance_phase());
        assert_eq!(state.phase, Phase::Plan);

        assert!(state.advance_phase());
        assert_eq!(state.phase, Phase::Run);

        assert!(state.advance_phase());
        assert_eq!(state.phase, Phase::Verify);

        assert!(!state.advance_phase());
        assert_eq!(state.phase, Phase::Verify);
    }

    #[test]
    fn test_should_increment_turns() {
        let mut state = RuntimeState::new();
        assert_eq!(state.turns, 0);

        state.increment_turn();
        assert_eq!(state.turns, 1);

        state.increment_turn();
        assert_eq!(state.turns, 2);
    }

    #[test]
    fn test_should_accumulate_cost() {
        let mut state = RuntimeState::new();
        assert_eq!(state.cost_usd, 0.0);

        state.add_cost(1.5);
        assert_eq!(state.cost_usd, 1.5);

        state.add_cost(2.3);
        assert_eq!(state.cost_usd, 3.8);
    }

    #[test]
    fn test_should_convert_phase_to_string() {
        assert_eq!(Phase::Init.as_str(), "init");
        assert_eq!(Phase::Plan.as_str(), "plan");
        assert_eq!(Phase::Run.as_str(), "run");
        assert_eq!(Phase::Verify.as_str(), "verify");
    }

    #[test]
    fn test_should_parse_phase_from_string() {
        assert_eq!("init".parse::<Phase>(), Ok(Phase::Init));
        assert_eq!("plan".parse::<Phase>(), Ok(Phase::Plan));
        assert_eq!("run".parse::<Phase>(), Ok(Phase::Run));
        assert_eq!("verify".parse::<Phase>(), Ok(Phase::Verify));
        assert!("invalid".parse::<Phase>().is_err());
    }

    #[test]
    fn test_should_display_phase() {
        assert_eq!(format!("{}", Phase::Init), "init");
        assert_eq!(format!("{}", Phase::Plan), "plan");
        assert_eq!(format!("{}", Phase::Run), "run");
        assert_eq!(format!("{}", Phase::Verify), "verify");
    }
}
