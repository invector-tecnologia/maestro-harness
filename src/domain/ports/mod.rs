//! Domain ports — trait contracts implemented by `infrastructure`.
//!
//! TASK 003 delivers `LlmProvider`. Later tasks add `Bus` (TASK 002) and the
//! `Role` cognitive contract (TASK 010). Traits here are pure and provider-agnostic.

pub mod llm_provider;
pub mod role;

pub use llm_provider::{
    CompletionRequest, CompletionResponse, LlmError, LlmProvider, ProviderStatus, TokenUsage,
};
pub use role::Role;
