//! Runtime observability (TASK 004).
//!
//! `RuntimeEvent` narrates the cognitive cycle so every collaborating agent
//! renders uniformly in the TUI. Emission always goes through `tracing`.

use crate::domain::models::AgentId;

/// A narrated step in an agent's cognitive cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEvent {
    /// The agent is registering incoming work (OBSERVE).
    AgentObserving { agent: AgentId },
    /// The agent is reasoning without side effects (THINK).
    AgentThinking { agent: AgentId },
    /// The agent is producing output (ACT).
    AgentActing { agent: AgentId },
    /// The agent reflected on its output (REFLECT phase).
    AgentReflected { agent: AgentId, satisfied: bool },
    /// The agent finished acting; `produced` indicates whether it emitted a message.
    AgentActed { agent: AgentId, produced: bool },
    /// The agent failed and was isolated (never silently dropped).
    AgentFailed { agent: AgentId, error: String },
}

impl RuntimeEvent {
    /// The agent this event concerns.
    pub fn agent(&self) -> &AgentId {
        match self {
            RuntimeEvent::AgentObserving { agent }
            | RuntimeEvent::AgentThinking { agent }
            | RuntimeEvent::AgentActing { agent }
            | RuntimeEvent::AgentReflected { agent, .. }
            | RuntimeEvent::AgentActed { agent, .. }
            | RuntimeEvent::AgentFailed { agent, .. } => agent,
        }
    }

    /// Emit the event through `tracing` with structured context.
    pub fn narrate(&self) {
        match self {
            RuntimeEvent::AgentObserving { agent } => {
                tracing::info!(agent = %agent, phase = "observe", "agent observing")
            }
            RuntimeEvent::AgentThinking { agent } => {
                tracing::info!(agent = %agent, phase = "think", "agent thinking")
            }
            RuntimeEvent::AgentActing { agent } => {
                tracing::info!(agent = %agent, phase = "act", "agent acting")
            }
            RuntimeEvent::AgentReflected { agent, satisfied } => {
                tracing::info!(agent = %agent, phase = "reflect", satisfied, "agent reflected")
            }
            RuntimeEvent::AgentActed { agent, produced } => {
                tracing::info!(agent = %agent, phase = "acted", produced, "agent acted")
            }
            RuntimeEvent::AgentFailed { agent, error } => {
                tracing::error!(agent = %agent, phase = "failed", error, "agent failed")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_agent_for_every_variant() {
        let id = AgentId::new("Maestro").unwrap();
        let events = [
            RuntimeEvent::AgentObserving { agent: id.clone() },
            RuntimeEvent::AgentThinking { agent: id.clone() },
            RuntimeEvent::AgentActing { agent: id.clone() },
            RuntimeEvent::AgentReflected {
                agent: id.clone(),
                satisfied: true,
            },
            RuntimeEvent::AgentActed {
                agent: id.clone(),
                produced: true,
            },
            RuntimeEvent::AgentFailed {
                agent: id.clone(),
                error: "boom".to_string(),
            },
        ];
        for event in events {
            assert_eq!(event.agent(), &id);
            event.narrate();
        }
    }
}
