//! Persona agents (TASK 011).
//!
//! A [`PersonaAgent`] wraps a [`Persona`] and an [`LlmProvider`], implementing the
//! `Role` cognitive contract. `activate_default_agents` brings the four
//! operational personas online (the orchestrator is excluded from the fan-out).

use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::models::{default_personas, AgentId, Message, Persona};
use crate::domain::ports::{CompletionRequest, LlmError, LlmProvider, Role};

/// A persona bound to a provider and model.
pub struct PersonaAgent {
    persona: Persona,
    provider: Arc<dyn LlmProvider>,
    model: String,
    inbox: Vec<Message>,
}

impl PersonaAgent {
    /// Bind a persona to a provider + model.
    pub fn new(persona: Persona, provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            persona,
            provider,
            model: model.into(),
            inbox: Vec::new(),
        }
    }
}

#[async_trait]
impl Role for PersonaAgent {
    fn id(&self) -> &AgentId {
        &self.persona.id
    }

    fn observe(&mut self, input: &[Message]) {
        self.inbox.extend_from_slice(input);
    }

    fn think(&mut self) {
        // Pure phase: no I/O. Working memory is already staged in `inbox`.
    }

    async fn act(&mut self) -> Result<Option<Message>, LlmError> {
        if self.inbox.is_empty() {
            return Ok(None);
        }
        let request = CompletionRequest {
            model: self.model.clone(),
            messages: std::mem::take(&mut self.inbox),
        };
        let response = self.provider.complete(request).await?;
        let message = Message::assistant(self.persona.id.clone(), response.content)
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
        Ok(Some(message))
    }
}

/// Bring the four operational personas online, bound to `provider`/`model`.
/// The orchestrator persona is excluded (it coordinates rather than fans out).
pub fn activate_default_agents(provider: Arc<dyn LlmProvider>, model: &str) -> Vec<Box<dyn Role>> {
    default_personas()
        .into_iter()
        .filter(|persona| !persona.orchestrator)
        .map(|persona| {
            Box::new(PersonaAgent::new(persona, provider.clone(), model)) as Box<dyn Role>
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::llm_provider::MockLlmProvider;
    use crate::domain::ports::{CompletionResponse, ProviderStatus};

    fn provider() -> Arc<dyn LlmProvider> {
        let mut mock = MockLlmProvider::new();
        mock.expect_probe().returning(|| ProviderStatus::Available);
        mock.expect_complete().returning(|_r| {
            Ok(CompletionResponse {
                content: "ok".to_string(),
            })
        });
        Arc::new(mock)
    }

    #[test]
    fn activates_four_operational_personas() {
        let agents = activate_default_agents(provider(), "mistral");
        assert_eq!(agents.len(), 4);
    }

    #[tokio::test]
    async fn empty_inbox_produces_nothing() {
        let persona = default_personas()
            .into_iter()
            .find(|p| !p.orchestrator)
            .unwrap();
        let mut agent = PersonaAgent::new(persona, provider(), "mistral");
        assert!(agent.act().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn acts_on_observed_input() {
        let persona = default_personas()
            .into_iter()
            .find(|p| !p.orchestrator)
            .unwrap();
        let mut agent = PersonaAgent::new(persona, provider(), "mistral");
        agent.observe(&[Message::user("go").unwrap()]);
        let out = agent.act().await.unwrap().unwrap();
        assert_eq!(out.content, "ok");
    }
}
