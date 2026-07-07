//! Application layer — use cases and orchestration.
//!
//! Owns agent lifecycle, the micro-project FSM, readiness (SENSE), and the
//! `observe -> think -> act` runtime narration. Depends on `domain` only.
//!
//! Populated by later tasks: `agent_runtime` (TASK 010), `agent_observability`
//! (TASK 004), `readiness` (TASK 010), and the FSM engine (TASK 046).

pub mod agent_observability;
pub mod agent_runtime;
pub mod error;
pub mod governance;
pub mod persona_agent;
pub mod readiness;
pub mod sops;
pub mod wizard;

pub use agent_observability::RuntimeEvent;
pub use agent_runtime::AgentRuntime;
pub use error::RuntimeError;
