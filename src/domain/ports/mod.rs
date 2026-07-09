//! Domain ports — trait contracts implemented by `infrastructure`.
//!
//! TASK 003 delivers `LlmProvider`. Later tasks add `Bus` (TASK 002) and the
//! `Role` cognitive contract (TASK 010). Traits here are pure and provider-agnostic.

pub mod llm_provider;
pub mod role;
pub mod scratchpad_port;
pub mod session_store;
pub mod tool_registry;

pub use llm_provider::{
    CompletionRequest, CompletionResponse, LlmError, LlmProvider, ProviderStatus, TokenUsage,
};
pub use role::Role;
pub use scratchpad_port::ScratchpadPort;
pub use session_store::{AgentTranscript, SessionStore, SessionStoreError, SessionTranscript};
#[cfg(test)]
pub use tool_registry::MockToolRegistry;
pub use tool_registry::ToolRegistry;
