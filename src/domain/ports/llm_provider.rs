//! `LlmProvider` — the domain port abstracting any LLM backend.
//!
//! TASK 003 (The AI Contract). Adapters live in `crate::infrastructure::llm`
//! (Ollama is the reference adapter, TASK 009). The registry/factory (TASK 008)
//! resolves a concrete provider behind `dyn LlmProvider`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::models::Message;

/// Result of probing a provider for readiness (the SENSE stage of the cognitive
/// pattern). Never blocks indefinitely; adapters apply timeouts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    /// Reachable and ready to serve the configured model.
    Available,
    /// The endpoint could not be reached (network/timeout).
    Unreachable,
    /// The endpoint rejected credentials.
    Unauthorized,
    /// The endpoint is reachable but the requested model is absent.
    ModelMissing,
}

/// A request for a single completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model identifier as declared in configuration.
    pub model: String,
    /// Ordered conversation context.
    pub messages: Vec<Message>,
}

/// A completion produced by a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The generated text.
    pub content: String,
    /// Token usage reported by the provider, if available.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub usage: Option<TokenUsage>,
}

/// Token counts reported by a provider for a single completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens consumed by the prompt/context.
    pub prompt_tokens: u64,
    /// Tokens generated in the completion.
    pub completion_tokens: u64,
}

/// Errors surfaced by a provider adapter. Adapters map transport/HTTP failures
/// into these typed variants — never panic.
#[derive(Debug, Error)]
pub enum LlmError {
    /// The provider could not be reached.
    #[error("provider unreachable: {0}")]
    Unreachable(String),
    /// Credentials were rejected.
    #[error("provider unauthorized")]
    Unauthorized,
    /// The requested model is not available.
    #[error("model not available: {0}")]
    ModelMissing(String),
    /// The provider returned a malformed or unexpected response.
    #[error("invalid provider response: {0}")]
    InvalidResponse(String),
}

/// The port every LLM backend implements. Object-safe via `async_trait` so the
/// registry can hold `Box<dyn LlmProvider>`.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Probe readiness for the SENSE stage. Must apply a timeout and degrade to a
    /// safe status rather than hanging.
    async fn probe(&self) -> ProviderStatus;

    /// Produce a single completion for the given request.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{AgentId, Message};

    #[tokio::test]
    async fn mock_provider_probes_and_completes() {
        let mut mock = MockLlmProvider::new();
        mock.expect_probe().returning(|| ProviderStatus::Available);
        mock.expect_complete().returning(|_req| {
            Ok(CompletionResponse {
                content: "ok".to_string(),
                usage: None,
            })
        });

        assert_eq!(mock.probe().await, ProviderStatus::Available);
        let req = CompletionRequest {
            model: "mistral".to_string(),
            messages: vec![Message::user("hi").unwrap()],
        };
        let resp = mock.complete(req).await.unwrap();
        assert_eq!(resp.content, "ok");
    }

    #[test]
    fn provider_status_serde_is_snake_case() {
        let json = serde_json::to_string(&ProviderStatus::ModelMissing).unwrap();
        assert_eq!(json, "\"model_missing\"");
    }

    #[test]
    fn completion_request_holds_context() {
        let author = AgentId::new("Maestro").unwrap();
        let req = CompletionRequest {
            model: "mistral".to_string(),
            messages: vec![Message::assistant(author, "plan").unwrap()],
        };
        assert_eq!(req.messages.len(), 1);
    }
}
