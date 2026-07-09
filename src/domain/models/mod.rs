//! Domain models — entities and value objects.
//!
//! TASK 001 (Core Domain Validation): `AgentId`, `MessageRole`, `Message`.
//! These types are pure: no I/O, no provider SDKs.

pub mod agent_id;
pub mod config;
pub mod fsm;
pub mod governance;
pub mod memory;
pub mod message;
pub mod persona;
pub mod reflection;
pub mod rollback;
pub mod routing;
pub mod thinking;

pub use agent_id::{AgentId, AgentIdError};
pub use config::{
    AgentBinding, ConfigError, MaestroConfig, ModelConfig, ProviderConfig, SystemConfig,
};
pub use fsm::{can_transition, FsmError, FsmStage, MicroProject};
pub use governance::{validate_entries, GovernanceReport, REQUIRED_GOVERNANCE_ENTRIES};
pub use memory::ShortTermMemory;
pub use message::{Message, MessageError, MessageRole};
pub use persona::{default_personas, Persona, PersonaError};
pub use reflection::ReflectionOutput;
pub use rollback::{CascadeStep, RollbackPlan};
pub use routing::{route, PersonaMatch, Routing};
pub use thinking::ThinkingOutput;
