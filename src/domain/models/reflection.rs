//! Structured output from an agent's REFLECT phase.
//!
//! After `act()`, the agent reviews its own output for quality concerns.

use serde::{Deserialize, Serialize};

/// The structured result of an agent's post-act self-critique.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectionOutput {
    /// Whether the agent is satisfied with its output.
    pub satisfied: bool,
    /// Quality concerns identified during reflection.
    pub concerns: Vec<String>,
    /// Suggested improvements (for future iterations or audit).
    pub suggestions: Vec<String>,
}
