//! Ollama reference adapter (TASK 009).
//!
//! Implements [`LlmProvider`] against a local Ollama endpoint using `reqwest`
//! with a timeout. Transport/HTTP failures map to typed [`LlmError`] /
//! [`ProviderStatus`] — never panics. Pure helpers (endpoint normalization, URL
//! building, status mapping, prompt rendering) are unit-tested without a network.

use std::time::Duration;

use async_trait::async_trait;

use crate::domain::models::{Message, MessageRole};
use crate::domain::ports::{
    CompletionRequest, CompletionResponse, LlmError, LlmProvider, ProviderStatus, TokenUsage,
};

/// An Ollama-backed provider.
pub struct OllamaProvider {
    base: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Build a provider for `endpoint` with the given request `timeout`.
    pub fn new(endpoint: &str, timeout: Duration) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| LlmError::Unreachable(e.to_string()))?;
        Ok(Self {
            base: normalize_endpoint(endpoint),
            client,
        })
    }

    fn tags_url(&self) -> String {
        format!("{}/api/tags", self.base)
    }

    fn generate_url(&self) -> String {
        format!("{}/api/generate", self.base)
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn probe(&self) -> ProviderStatus {
        match self.client.get(self.tags_url()).send().await {
            Ok(resp) => probe_status_from_code(resp.status().as_u16()),
            Err(_) => ProviderStatus::Unreachable,
        }
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let body = serde_json::json!({
            "model": request.model,
            "prompt": render_prompt(&request.messages),
            "stream": false,
        });
        let resp = self
            .client
            .post(self.generate_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Unreachable(e.to_string()))?;

        match resp.status().as_u16() {
            200 => {
                let value: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
                let content = value
                    .get("response")
                    .and_then(|r| r.as_str())
                    .ok_or_else(|| LlmError::InvalidResponse("missing 'response' field".into()))?;
                let usage = parse_ollama_usage(&value);
                Ok(CompletionResponse {
                    content: content.to_string(),
                    usage,
                })
            }
            401 | 403 => Err(LlmError::Unauthorized),
            404 => Err(LlmError::ModelMissing(request.model)),
            other => Err(LlmError::Unreachable(format!("unexpected status {other}"))),
        }
    }
}

/// Strip an OpenAI-compat `/v1` suffix and trailing slashes so native `/api/*`
/// paths can be appended cleanly.
pub fn normalize_endpoint(endpoint: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    let without_v1 = trimmed.strip_suffix("/v1").unwrap_or(trimmed);
    without_v1.trim_end_matches('/').to_string()
}

/// Parse the token usage from an Ollama JSON response.
fn parse_ollama_usage(value: &serde_json::Value) -> Option<TokenUsage> {
    let prompt = value.get("prompt_eval_count")?.as_u64()?;
    let completion = value.get("eval_count")?.as_u64()?;
    Some(TokenUsage {
        prompt_tokens: prompt,
        completion_tokens: completion,
    })
}

/// Map an HTTP status code from the tags probe to a [`ProviderStatus`].
pub fn probe_status_from_code(code: u16) -> ProviderStatus {
    match code {
        200 => ProviderStatus::Available,
        401 | 403 => ProviderStatus::Unauthorized,
        404 => ProviderStatus::ModelMissing,
        _ => ProviderStatus::Unreachable,
    }
}

/// Render ordered messages into a single prompt string.
pub fn render_prompt(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|m| {
            let role = match m.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };
            format!("{role}: {}", m.content)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_v1_and_trailing_slash() {
        assert_eq!(
            normalize_endpoint("http://127.0.0.1:11434/v1"),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            normalize_endpoint("http://localhost:11434/"),
            "http://localhost:11434"
        );
    }

    #[test]
    fn builds_native_api_urls() {
        let p = OllamaProvider::new("http://127.0.0.1:11434/v1", Duration::from_secs(5)).unwrap();
        assert_eq!(p.tags_url(), "http://127.0.0.1:11434/api/tags");
        assert_eq!(p.generate_url(), "http://127.0.0.1:11434/api/generate");
    }

    #[test]
    fn maps_status_codes() {
        assert_eq!(probe_status_from_code(200), ProviderStatus::Available);
        assert_eq!(probe_status_from_code(401), ProviderStatus::Unauthorized);
        assert_eq!(probe_status_from_code(404), ProviderStatus::ModelMissing);
        assert_eq!(probe_status_from_code(500), ProviderStatus::Unreachable);
    }

    #[test]
    fn renders_prompt_in_order() {
        let msgs = vec![
            Message::system("be brief").unwrap(),
            Message::user("hi").unwrap(),
        ];
        assert_eq!(render_prompt(&msgs), "system: be brief\nuser: hi");
    }

    #[test]
    fn parses_usage_when_present() {
        let json = serde_json::json!({
            "response": "ok",
            "prompt_eval_count": 10,
            "eval_count": 20
        });
        let usage = parse_ollama_usage(&json).unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
    }

    #[test]
    fn missing_usage_returns_none() {
        let json = serde_json::json!({
            "response": "ok"
        });
        assert!(parse_ollama_usage(&json).is_none());
    }
}
