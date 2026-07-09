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
    /// An agent published a message to the inter-agent bus.
    AgentPublished { agent: AgentId },
    /// An agent sent a directed message to a specific peer.
    AgentDirectedSend { sender: AgentId, recipient: AgentId },
    /// An agent wrote to the shared scratchpad.
    ScratchpadWrite { agent: AgentId, key: String },
    /// An agent's lifecycle status changed.
    AgentLifecycle { agent: AgentId, status: String },
    /// A per-agent metrics snapshot emitted after a cycle.
    AgentMetricsSnapshot {
        agent: AgentId,
        cycles: u64,
        successes: u64,
        failures: u64,
        prompt_tokens: u64,
        completion_tokens: u64,
        latency_ms: u64,
    },
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
            | RuntimeEvent::AgentFailed { agent, .. }
            | RuntimeEvent::AgentPublished { agent, .. }
            | RuntimeEvent::AgentDirectedSend { sender: agent, .. }
            | RuntimeEvent::ScratchpadWrite { agent, .. }
            | RuntimeEvent::AgentLifecycle { agent, .. }
            | RuntimeEvent::AgentMetricsSnapshot { agent, .. } => agent,
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
            RuntimeEvent::AgentPublished { agent } => {
                tracing::info!(agent = %agent, phase = "published", "agent published message")
            }
            RuntimeEvent::AgentDirectedSend { sender, recipient } => {
                tracing::info!(agent = %sender, recipient = %recipient, phase = "directed_send", "agent sent directed message")
            }
            RuntimeEvent::ScratchpadWrite { agent, key } => {
                tracing::info!(agent = %agent, key = %key, phase = "scratchpad_write", "agent wrote to scratchpad")
            }
            RuntimeEvent::AgentLifecycle { agent, status } => {
                tracing::info!(agent = %agent, phase = "lifecycle", status, "agent lifecycle event")
            }
            RuntimeEvent::AgentMetricsSnapshot {
                agent,
                cycles,
                successes,
                failures,
                prompt_tokens,
                completion_tokens,
                latency_ms,
            } => {
                tracing::info!(
                    agent = %agent,
                    phase = "metrics",
                    cycles,
                    successes,
                    failures,
                    prompt_tokens,
                    completion_tokens,
                    latency_ms,
                    "agent metrics snapshot"
                )
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
            RuntimeEvent::AgentPublished { agent: id.clone() },
            RuntimeEvent::AgentDirectedSend {
                sender: id.clone(),
                recipient: AgentId::new("Peer").unwrap(),
            },
            RuntimeEvent::ScratchpadWrite {
                agent: id.clone(),
                key: "notes".to_string(),
            },
            RuntimeEvent::AgentLifecycle {
                agent: id.clone(),
                status: "Running".to_string(),
            },
            RuntimeEvent::AgentMetricsSnapshot {
                agent: id.clone(),
                cycles: 1,
                successes: 1,
                failures: 0,
                prompt_tokens: 10,
                completion_tokens: 20,
                latency_ms: 100,
            },
        ];
        for event in events {
            assert_eq!(event.agent(), &id);
            event.narrate();
        }
    }
}
