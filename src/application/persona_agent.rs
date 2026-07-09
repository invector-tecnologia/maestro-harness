//! Persona agents (TASK 011).
//!
//! A [`PersonaAgent`] wraps a [`Persona`] and an [`LlmProvider`], implementing the
//! `Role` cognitive contract. `activate_default_agents` brings the four
//! operational personas online (the orchestrator is excluded from the fan-out).

use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::models::{
    default_personas, AgentId, Message, Persona, ReflectionOutput, ShortTermMemory, ThinkingOutput,
};
use crate::domain::ports::{CompletionRequest, LlmError, LlmProvider, Role};

/// A persona bound to a provider and model.
pub struct PersonaAgent {
    persona: Persona,
    provider: Arc<dyn LlmProvider>,
    model: String,
    inbox: Vec<Message>,
    last_thinking: Option<ThinkingOutput>,
    memory: ShortTermMemory,
}

impl PersonaAgent {
    /// Bind a persona to a provider + model.
    pub fn new(persona: Persona, provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            persona,
            provider,
            model: model.into(),
            inbox: Vec::new(),
            last_thinking: None,
            memory: ShortTermMemory::new(32),
        }
    }

    /// Build the system prompt from the persona's identity.
    fn system_prompt(&self) -> String {
        if !self.persona.system_prompt.is_empty() {
            self.persona.system_prompt.clone()
        } else {
            format!(
                "You are '{}'. Your responsibility: {}\n\n\
                 Follow a structured approach:\n\
                 1. Interpret the task in terms of your specific role.\n\
                 2. Apply your expertise to produce a focused, actionable contribution.\n\
                 3. Flag any risks or concerns within your domain.\n\
                 4. Stay within your responsibility boundary — delegate what is outside it.",
                self.persona.id, self.persona.responsibility
            )
        }
    }

    /// Heuristic: does the demand overlap with this persona's responsibility keywords?
    fn assess_competence(&self, demand: &str) -> bool {
        let responsibility_lower = self.persona.responsibility.to_lowercase();
        let demand_lower = demand.to_lowercase();
        // Simple word-overlap heuristic
        responsibility_lower
            .split_whitespace()
            .filter(|w| w.len() > 3) // skip articles/prepositions
            .any(|word| demand_lower.contains(word))
    }
}

#[async_trait]
impl Role for PersonaAgent {
    fn id(&self) -> &AgentId {
        &self.persona.id
    }

    fn observe(&mut self, input: &[Message]) {
        self.inbox.extend_from_slice(input);
        for msg in input {
            self.memory.record(msg.clone());
        }
    }

    fn think(&mut self) -> ThinkingOutput {
        let combined_input: String = self
            .inbox
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let within_competence = self.assess_competence(&combined_input);

        let output = ThinkingOutput {
            task_interpretation: format!(
                "As {}, I interpret this task through the lens of: {}",
                self.persona.id, self.persona.responsibility
            ),
            approach: format!(
                "I will apply my expertise in '{}' to produce a focused contribution.",
                self.persona.responsibility
            ),
            risks: if within_competence {
                Vec::new()
            } else {
                vec!["This task may be partially outside my primary responsibility.".to_string()]
            },
            within_competence,
        };

        self.last_thinking = Some(output.clone());
        output
    }

    async fn act(&mut self) -> Result<Option<Message>, LlmError> {
        if self.inbox.is_empty() {
            return Ok(None);
        }

        let mut messages = Vec::new();

        // 1. System prompt (persona identity)
        if let Ok(sys) = Message::system(self.system_prompt()) {
            messages.push(sys);
        }

        // 2. Thinking output as context
        if let Some(ref thinking) = self.last_thinking {
            if let Ok(ctx) = Message::system(format!(
                "[Internal Reasoning]\n{}",
                thinking.as_prompt_fragment()
            )) {
                messages.push(ctx);
            }
        }

        // 3. Memory context (prior cycles)
        let memory_msgs: Vec<_> = self
            .memory
            .messages()
            .iter()
            .filter(|m| !self.inbox.contains(m))
            .cloned()
            .collect();
        messages.extend(memory_msgs);

        // 4. The observed conversation (current cycle)
        messages.extend(std::mem::take(&mut self.inbox));

        let request = CompletionRequest {
            model: self.model.clone(),
            messages,
        };
        let response = self.provider.complete(request).await?;
        self.last_thinking = None;
        let message = Message::assistant(self.persona.id.clone(), response.content)
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
        Ok(Some(message))
    }

    fn reflect(&self, output: &Message) -> ReflectionOutput {
        let mut concerns = Vec::new();
        let mut suggestions = Vec::new();

        // Heuristic 1: Very short responses may lack substance
        if output.content.len() < 20 {
            concerns.push("Response is very short — may lack actionable detail.".to_string());
            suggestions.push("Consider elaborating with specific steps or examples.".to_string());
        }

        // Heuristic 2: Very long responses may lack focus
        if output.content.len() > 5000 {
            concerns.push("Response is very long — may lack focus.".to_string());
            suggestions.push("Consider condensing to key actionable points.".to_string());
        }

        ReflectionOutput {
            satisfied: concerns.is_empty(),
            concerns,
            suggestions,
        }
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
    fn activates_seven_operational_personas() {
        let agents = activate_default_agents(provider(), "mistral");
        assert_eq!(agents.len(), 7);
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
