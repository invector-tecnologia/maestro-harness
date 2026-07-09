//! Gemini API provider adapter.
//!
//! Implements [`LlmProvider`] against the Google AI Gemini API.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::domain::models::MessageRole;
use crate::domain::ports::{
    CompletionRequest, CompletionResponse, LlmError, LlmProvider, ProviderStatus, TokenUsage,
};

/// A Gemini API provider.
pub struct GeminiProvider {
    base: String,
    api_key: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    /// Build a provider. Endpoint defaults to `https://generativelanguage.googleapis.com` if empty.
    pub fn new(
        endpoint: &str,
        api_key: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| LlmError::Unreachable(e.to_string()))?;

        let base = if endpoint.trim().is_empty() {
            "https://generativelanguage.googleapis.com".to_string()
        } else {
            normalize_base(endpoint)
        };

        Ok(Self {
            base,
            api_key: api_key.into(),
            client,
        })
    }

    fn model_url(&self, model: &str) -> String {
        format!("{}/v1beta/models/{}?key={}", self.base, model, self.api_key)
    }

    fn generate_url(&self, model: &str) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base, model, self.api_key
        )
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn probe(&self) -> ProviderStatus {
        if self.api_key.is_empty() {
            return ProviderStatus::Unauthorized;
        }
        match self
            .client
            .get(self.model_url("gemini-1.5-flash"))
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

        let body = build_gemini_body(&request).map_err(|e| {
            LlmError::InvalidResponse(format!("failed to serialize request: {}", e))
        })?;
        let resp = self
            .client
            .post(self.generate_url(&request.model))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Unreachable(e.to_string()))?;

        match resp.status().as_u16() {
            200 => {
                let value: GeminiResponse = resp
                    .json()
                    .await
                    .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

                let content = value
                    .candidates
                    .first()
                    .and_then(|c| c.content.parts.first())
                    .map(|p| p.text.clone())
                    .ok_or_else(|| {
                        LlmError::InvalidResponse(
                            "missing candidates[0].content.parts[0].text".into(),
                        )
                    })?;

                let usage = value.usage_metadata.as_ref().and_then(parse_gemini_usage);

                Ok(CompletionResponse { content, usage })
            }
            401 | 403 => Err(LlmError::Unauthorized),
            404 => Err(LlmError::ModelMissing(request.model)),
            other => Err(LlmError::Unreachable(format!("unexpected status {other}"))),
        }
    }
}

pub fn normalize_base(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

fn parse_gemini_usage(meta: &UsageMetadata) -> Option<TokenUsage> {
    Some(TokenUsage {
        prompt_tokens: meta.prompt_token_count,
        completion_tokens: meta.candidates_token_count,
    })
}

pub fn status_from_code(code: u16) -> ProviderStatus {
    match code {
        200 => ProviderStatus::Available,
        400 | 401 | 403 => ProviderStatus::Unauthorized,
        404 => ProviderStatus::ModelMissing,
        _ => ProviderStatus::Unreachable,
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Serialize, PartialEq, Debug)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount", default)]
    prompt_token_count: u64,
    #[serde(rename = "candidatesTokenCount", default)]
    candidates_token_count: u64,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    #[serde(default)]
    parts: Vec<Part>,
}

pub fn build_gemini_body(
    request: &CompletionRequest,
) -> Result<serde_json::Value, serde_json::Error> {
    let mut system_texts = Vec::new();
    let mut contents = Vec::new();

    for msg in &request.messages {
        match msg.role {
            MessageRole::System => system_texts.push(msg.content.clone()),
            MessageRole::User => contents.push(Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: msg.content.clone(),
                }],
            }),
            MessageRole::Assistant => contents.push(Content {
                role: "model".to_string(),
                parts: vec![Part {
                    text: msg.content.clone(),
                }],
            }),
        }
    }

    let system_instruction = if system_texts.is_empty() {
        None
    } else {
        Some(SystemInstruction {
            parts: vec![Part {
                text: system_texts.join("\n"),
            }],
        })
    };

    let req = GeminiRequest {
        system_instruction,
        contents,
    };

    serde_json::to_value(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{AgentId, Message};

    #[test]
    fn normalizes_base_trailing_slash() {
        assert_eq!(
            normalize_base("https://generativelanguage.googleapis.com/"),
            "https://generativelanguage.googleapis.com"
        );
    }

    #[test]
    fn builds_urls() {
        let p = GeminiProvider::new(
            "https://generativelanguage.googleapis.com",
            "KEY",
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(
            p.model_url("gemini-1.5-flash"),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash?key=KEY"
        );
        assert_eq!(p.generate_url("gemini-1.5-pro"), "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro:generateContent?key=KEY");
    }

    #[test]
    fn maps_status_codes() {
        assert_eq!(status_from_code(200), ProviderStatus::Available);
        assert_eq!(status_from_code(400), ProviderStatus::Unauthorized);
        assert_eq!(status_from_code(401), ProviderStatus::Unauthorized);
        assert_eq!(status_from_code(403), ProviderStatus::Unauthorized);
        assert_eq!(status_from_code(404), ProviderStatus::ModelMissing);
        assert_eq!(status_from_code(503), ProviderStatus::Unreachable);
    }

    #[test]
    fn chat_body_maps_roles_correctly() {
        let request = CompletionRequest {
            model: "gemini-1.5-flash".to_string(),
            messages: vec![
                Message::system("be brief").unwrap(),
                Message::user("hi").unwrap(),
                Message::assistant(AgentId::new("Maestro").unwrap(), "hello").unwrap(),
            ],
        };
        let body = build_gemini_body(&request).unwrap();

        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "be brief");
        assert_eq!(body["contents"][0]["role"], "user");
        assert_eq!(body["contents"][0]["parts"][0]["text"], "hi");
        assert_eq!(body["contents"][1]["role"], "model");
        assert_eq!(body["contents"][1]["parts"][0]["text"], "hello");
    }

    #[test]
    fn parses_usage_when_present() {
        let meta = UsageMetadata {
            prompt_token_count: 50,
            candidates_token_count: 30,
        };
        let usage = parse_gemini_usage(&meta).unwrap();
        assert_eq!(usage.prompt_tokens, 50);
        assert_eq!(usage.completion_tokens, 30);
    }
}
