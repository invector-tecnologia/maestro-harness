//! Structured output from an agent's THINK phase.
//!
//! `ThinkingOutput` captures the agent's local reasoning before acting:
//! task decomposition, approach selection, and risk flagging. It is
//! injected into the conversation context so the LLM receives structured
//! guidance alongside the raw user input.

use serde::{Deserialize, Serialize};

/// The structured result of an agent's `think()` phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingOutput {
    /// How the agent interprets the task in terms of its responsibility.
    pub task_interpretation: String,
    /// The approach the agent will take (methodology, constraints).
    pub approach: String,
    /// Risks or concerns the agent identified.
    pub risks: Vec<String>,
    /// Whether the agent considers this task within its competence.
    pub within_competence: bool,
}

impl ThinkingOutput {
    /// Render the thinking output as a structured prompt fragment
    /// suitable for injection into the LLM conversation context.
    pub fn as_prompt_fragment(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "**Task Interpretation:** {}",
            self.task_interpretation
        ));
        lines.push(format!("**Approach:** {}", self.approach));
        if !self.risks.is_empty() {
            lines.push(format!("**Risks:** {}", self.risks.join("; ")));
        }
        if !self.within_competence {
            lines.push("**Note:** This task may be outside my primary competence.".to_string());
        }
        lines.join("\n")
    }
}
