//! Micro-project finite state machine (TASK 046).
//!
//! The six-stage lifecycle every micro-project flows through:
//! `Ideation → Planning → Approval → Instrumentation → Execution → Verification`.
//! Transitions are strictly sequential; illegal transitions are rejected with a
//! typed error. Pure domain logic — instrumentation/IPC lives in the application
//! and presentation layers.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A stage in the micro-project lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsmStage {
    /// Capture the user request.
    Ideation,
    /// Produce the strategic plan.
    Planning,
    /// User validates the plan (gated by IPC approval).
    Approval,
    /// Generate personas + inject prompts/skills/specs.
    Instrumentation,
    /// Serial cascade of actions (gated by rollback plan + approval).
    Execution,
    /// Validate the outcome, package, and persist.
    Verification,
}

impl FsmStage {
    /// The ordered lifecycle.
    pub const ALL: [FsmStage; 6] = [
        FsmStage::Ideation,
        FsmStage::Planning,
        FsmStage::Approval,
        FsmStage::Instrumentation,
        FsmStage::Execution,
        FsmStage::Verification,
    ];

    /// The snake_case label used on the IPC boundary and in narration.
    pub fn as_str(self) -> &'static str {
        match self {
            FsmStage::Ideation => "ideation",
            FsmStage::Planning => "planning",
            FsmStage::Approval => "approval",
            FsmStage::Instrumentation => "instrumentation",
            FsmStage::Execution => "execution",
            FsmStage::Verification => "verification",
        }
    }

    /// The next stage in sequence, or `None` at the terminal `Verification`.
    pub fn next(self) -> Option<FsmStage> {
        match self {
            FsmStage::Ideation => Some(FsmStage::Planning),
            FsmStage::Planning => Some(FsmStage::Approval),
            FsmStage::Approval => Some(FsmStage::Instrumentation),
            FsmStage::Instrumentation => Some(FsmStage::Execution),
            FsmStage::Execution => Some(FsmStage::Verification),
            FsmStage::Verification => None,
        }
    }

    /// Whether this stage gates on explicit user approval before advancing.
    pub fn requires_approval(self) -> bool {
        matches!(self, FsmStage::Approval | FsmStage::Execution)
    }
}

/// A transition that violates the sequential lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FsmError {
    /// The requested transition is not the legal next step.
    #[error("illegal FSM transition from {from} to {to}")]
    IllegalTransition {
        /// Current stage label.
        from: String,
        /// Requested stage label.
        to: String,
    },
    /// Advance was requested past the terminal stage.
    #[error("cannot advance past the terminal stage {0}")]
    Terminal(String),
}

/// Whether `to` is the legal next stage after `from`.
pub fn can_transition(from: FsmStage, to: FsmStage) -> bool {
    from.next() == Some(to)
}

/// A micro-project and its current lifecycle stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicroProject {
    /// Stable id for the run.
    pub id: String,
    /// The originating user demand.
    pub demand: String,
    /// The current lifecycle stage.
    pub stage: FsmStage,
}

impl MicroProject {
    /// A fresh micro-project in `Ideation`.
    pub fn new(id: impl Into<String>, demand: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            demand: demand.into(),
            stage: FsmStage::Ideation,
        }
    }

    /// Advance to the next stage, returning it. Errors at the terminal stage.
    pub fn advance(&mut self) -> Result<FsmStage, FsmError> {
        match self.stage.next() {
            Some(next) => {
                self.stage = next;
                Ok(next)
            }
            None => Err(FsmError::Terminal(self.stage.as_str().to_string())),
        }
    }

    /// Transition to an explicit stage, rejecting illegal jumps.
    pub fn transition_to(&mut self, to: FsmStage) -> Result<(), FsmError> {
        if can_transition(self.stage, to) {
            self.stage = to;
            Ok(())
        } else {
            Err(FsmError::IllegalTransition {
                from: self.stage.as_str().to_string(),
                to: to.as_str().to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_transitions_follow_the_sequence() {
        for pair in FsmStage::ALL.windows(2) {
            assert!(can_transition(pair[0], pair[1]));
        }
    }

    #[test]
    fn illegal_transitions_are_rejected() {
        assert!(!can_transition(FsmStage::Ideation, FsmStage::Approval));
        assert!(!can_transition(FsmStage::Execution, FsmStage::Planning));
        assert!(!can_transition(FsmStage::Verification, FsmStage::Ideation));
    }

    #[test]
    fn advance_walks_the_full_lifecycle_then_errors() {
        let mut mp = MicroProject::new("mp-1", "build a cli");
        assert_eq!(mp.stage, FsmStage::Ideation);
        for expected in &FsmStage::ALL[1..] {
            assert_eq!(mp.advance().unwrap(), *expected);
        }
        assert_eq!(mp.stage, FsmStage::Verification);
        assert!(matches!(mp.advance(), Err(FsmError::Terminal(_))));
    }

    #[test]
    fn transition_to_rejects_illegal_jump() {
        let mut mp = MicroProject::new("mp-2", "x");
        assert!(matches!(
            mp.transition_to(FsmStage::Execution),
            Err(FsmError::IllegalTransition { .. })
        ));
        assert_eq!(mp.stage, FsmStage::Ideation);
    }

    #[test]
    fn approval_and_execution_are_gated() {
        assert!(FsmStage::Approval.requires_approval());
        assert!(FsmStage::Execution.requires_approval());
        assert!(!FsmStage::Planning.requires_approval());
    }
}
