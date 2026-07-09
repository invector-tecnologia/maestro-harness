//! `Role` — the cognitive contract every agent implements (TASK 010).
//!
//! The innermost loop of the canonical pattern: `OBSERVE → THINK → ACT → REFLECT`.
//! `think` is intentionally synchronous and side-effect-free (no I/O); all
//! provider I/O and output happen in `act`. `reflect` is a synchronous
//! post-act self-critique.

use async_trait::async_trait;

use crate::domain::models::{AgentId, Message, ReflectionOutput, ThinkingOutput};

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
    /// Returns structured reasoning that will be injected into the act() context.
    fn think(&mut self) -> ThinkingOutput;

    /// ACT: produce an optional output message (may perform provider I/O).
    async fn act(&mut self) -> Result<Option<Message>, LlmError>;

    /// REFLECT: review the output of act() for quality concerns.
    /// Pure — no I/O. Called only when act() produced a message.
    fn reflect(&self, output: &Message) -> ReflectionOutput;
}
