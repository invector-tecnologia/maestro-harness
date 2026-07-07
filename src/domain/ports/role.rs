//! `Role` — the cognitive contract every agent implements (TASK 010).
//!
//! The innermost loop of the canonical pattern: `OBSERVE → THINK → ACT`.
//! `think` is intentionally synchronous and side-effect-free (no I/O); all
//! provider I/O and output happen in `act`.

use async_trait::async_trait;

use crate::domain::models::{AgentId, Message};

use super::LlmError;

/// An agent's cognitive contract.
#[async_trait]
pub trait Role: Send + Sync {
    /// Stable identity of this agent.
    fn id(&self) -> &AgentId;

    /// OBSERVE: register incoming messages as the current unit of work.
    fn observe(&mut self, input: &[Message]);

    /// THINK: reason about observed input. Must be pure — no I/O, no external
    /// state mutation beyond the agent's own working memory.
    fn think(&mut self);

    /// ACT: produce an optional output message (may perform provider I/O).
    async fn act(&mut self) -> Result<Option<Message>, LlmError>;
}
