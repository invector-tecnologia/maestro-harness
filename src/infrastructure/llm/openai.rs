//! OpenAI-compatible provider adapter (TASK — Phase 6 / W6).
//!
//! Implements [`LlmProvider`] against any OpenAI-compatible Chat Completions API
//! (`/models` probe, `/chat/completions`). The API key is supplied at
//! construction (the registry reads it from the environment). Transport/HTTP
//! failures map to typed [`LlmError`] / [`ProviderStatus`] — never panics.

use std::time::Duration;

use async_trait::async_trait;

use crate::domain::models::MessageRole;
use crate::domain::ports::{
    CompletionRequest, CompletionResponse, LlmError, LlmProvider, ProviderStatus, TokenUsage,
};

/// An OpenAI-compatible provider (OpenAI and API-compatible gateways).
pub struct OpenAiProvider {
    base: String,
    api_key: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    /// Build a provider for `endpoint` (e.g. `https://api.openai.com/v1`) with an
    /// API key and per-request `timeout`.
    pub fn new(
        endpoint: &str,
        api_key: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| LlmError::Unreachable(e.to_string()))?;
        Ok(Self {
            base: normalize_base(endpoint),
            api_key: api_key.into(),
            client,
        })
    }

    fn models_url(&self) -> String {
        format!("{}/models", self.base)
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base)
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn probe(&self) -> ProviderStatus {
        if self.api_key.is_empty() {
            return ProviderStatus::Unauthorized;
        }
        match self
            .client
            .get(self.models_url())
            .bearer_auth(&self.api_key)
            .send()
            .await
        {
            Ok(resp) => status_from_code(resp.status().as_u16()),
            Err(_) => ProviderStatus::Unreachable,
        }
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        if self.api_key.is_empty() {
            return Err(LlmError::Unauthorized);
        }
        let resp = self
            .client
            .post(self.chat_url())
            .bearer_auth(&self.api_key)
            .json(&build_chat_body(&request))
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
                    .pointer("/choices/0/message/content")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| {
                        LlmError::InvalidResponse("missing choices[0].message.content".into())
                    })?;
                let usage = parse_openai_usage(&value);
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

/// Trim trailing slashes from the base URL (keep any `/v1` — OpenAI needs it).
pub fn normalize_base(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

/// Parse token usage from the OpenAI JSON response.
fn parse_openai_usage(value: &serde_json::Value) -> Option<TokenUsage> {
    let usage = value.get("usage")?;
    let prompt = usage.get("prompt_tokens")?.as_u64()?;
    let completion = usage.get("completion_tokens")?.as_u64()?;
    Some(TokenUsage {
        prompt_tokens: prompt,
        completion_tokens: completion,
    })
}

/// Map an HTTP status code to a [`ProviderStatus`].
pub fn status_from_code(code: u16) -> ProviderStatus {
    match code {
        200 => ProviderStatus::Available,
        401 | 403 => ProviderStatus::Unauthorized,
        404 => ProviderStatus::ModelMissing,
        _ => ProviderStatus::Unreachable,
    }
}

/// The OpenAI role label for a message role.
fn role_str(role: MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
    }
}

/// Build the Chat Completions request body.
pub fn build_chat_body(request: &CompletionRequest) -> serde_json::Value {
    let messages: Vec<serde_json::Value> = request
        .messages
        .iter()
        .map(|m| serde_json::json!({ "role": role_str(m.role), "content": m.content }))
        .collect();
    serde_json::json!({ "model": request.model, "messages": messages })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Message;

    #[test]
    fn normalizes_base_trailing_slash() {
        assert_eq!(
            normalize_base("https://api.openai.com/v1/"),
            "https://api.openai.com/v1"
        );
    }

    #[test]
    fn builds_models_and_chat_urls() {
        let p = OpenAiProvider::new(
            "https://api.openai.com/v1",
            "sk-test",
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(p.models_url(), "https://api.openai.com/v1/models");
        assert_eq!(p.chat_url(), "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn maps_status_codes() {
        assert_eq!(status_from_code(200), ProviderStatus::Available);
        assert_eq!(status_from_code(401), ProviderStatus::Unauthorized);
        assert_eq!(status_from_code(404), ProviderStatus::ModelMissing);
        assert_eq!(status_from_code(503), ProviderStatus::Unreachable);
    }

    #[test]
    fn chat_body_has_model_and_roled_messages() {
        let request = CompletionRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                Message::system("be brief").unwrap(),
                Message::user("hi").unwrap(),
            ],
        };
        let body = build_chat_body(&request);
        assert_eq!(body["model"], "gpt-4o-mini");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["role"], "user");
        assert_eq!(body["messages"][1]["content"], "hi");
    }

    #[tokio::test]
    async fn missing_key_is_unauthorized() {
        let p =
            OpenAiProvider::new("https://api.openai.com/v1", "", Duration::from_secs(5)).unwrap();
        assert_eq!(p.probe().await, ProviderStatus::Unauthorized);
        let request = CompletionRequest {
            model: "gpt-4o".to_string(),
            messages: vec![Message::user("hi").unwrap()],
        };
        assert!(matches!(
            p.complete(request).await,
            Err(LlmError::Unauthorized)
        ));
    }

    #[test]
    fn parses_usage_when_present() {
        let json = serde_json::json!({
            "usage": {
                "prompt_tokens": 15,
                "completion_tokens": 25
            }
        });
        let usage = parse_openai_usage(&json).unwrap();
        assert_eq!(usage.prompt_tokens, 15);
        assert_eq!(usage.completion_tokens, 25);
    }
}
