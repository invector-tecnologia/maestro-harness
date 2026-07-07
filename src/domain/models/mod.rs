//! Domain models — entities and value objects.
//!
//! TASK 001 (Core Domain Validation): `AgentId`, `MessageRole`, `Message`.
//! These types are pure: no I/O, no provider SDKs.

pub mod agent_id;
pub mod config;
pub mod governance;
pub mod message;
pub mod persona;

pub use agent_id::{AgentId, AgentIdError};
pub use config::{
    AgentBinding, ConfigError, MaestroConfig, ModelConfig, ProviderConfig, SystemConfig,
};
pub use governance::{validate_entries, GovernanceReport, REQUIRED_GOVERNANCE_ENTRIES};
pub use message::{Message, MessageError, MessageRole};
pub use persona::{default_personas, Persona, PersonaError};
