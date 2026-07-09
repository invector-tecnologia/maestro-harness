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
use crate::domain::ports::{CompletionRequest, LlmError, LlmProvider, Role, TokenUsage};

/// A persona bound to a provider and model.
pub struct PersonaAgent {
    persona: Persona,
    provider: Arc<dyn LlmProvider>,
    model: String,
    inbox: Vec<Message>,
    last_thinking: Option<ThinkingOutput>,
    last_usage: Option<TokenUsage>,
    memory: ShortTermMemory,
    tools: Option<Arc<dyn crate::domain::ports::ToolRegistry>>,
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
            last_usage: None,
            memory: ShortTermMemory::new(32),
            tools: None,
        }
    }

    /// Attach a tool registry to this agent.
    pub fn with_tools(mut self, tools: Arc<dyn crate::domain::ports::ToolRegistry>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Build the system prompt from the persona's identity.
    fn system_prompt(&self) -> String {
        let mut prompt = format!(
            "You are {}, {}.\n\nYour mandate:\n{}\n\nConstraints:\n{}\n\n{}",
            self.persona.id,
            self.persona.responsibility,
            self.persona.responsibility,
            "Stay within your responsibility boundary — delegate what is outside it.",
            self.persona.system_prompt
        );

        prompt.push_str("\n\n## Directed Messaging\n");
        prompt.push_str("To send a message to a specific agent, use:\n");
        prompt.push_str("[SEND_TO agent_name]\n");
        prompt.push_str("Your message content here.\n");
        prompt.push_str("[/SEND_TO]\n");

        if let Some(ref tools) = self.tools {
            prompt.push_str("\n\n## Available Tools\n");
            prompt.push_str("To use a tool, include a [TOOL_CALL]...[/TOOL_CALL] block:\n\n");
            for tool in tools.available_tools() {
                prompt.push_str(&format!(
                    "- **{}**: {} {}\n",
                    tool.name,
                    tool.description,
                    if tool.requires_approval {
                        "(requires approval)"
                    } else {
                        ""
                    }
                ));
            }
            prompt.push_str("\nExample:\n");
            prompt.push_str("[TOOL_CALL]\n{\"tool\": \"read_file\", \"arguments\": {\"path\": \"src/main.rs\"}}\n[/TOOL_CALL]\n");
        }
        prompt
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

    /// Hydrate memory from a prior session transcript.
    pub fn hydrate(&mut self, messages: &[Message]) {
        self.memory.hydrate(messages);
    }

    /// Export the current memory for persistence.
    pub fn export_memory(&self) -> (Vec<Message>, Option<String>) {
        (
            self.memory.export(),
            self.memory.summary().map(|s| s.to_string()),
        )
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

        // 2b. Eviction summary from prior cycles
        if let Some(summary) = self.memory.summary() {
            if let Ok(ctx) =
                Message::system(format!("[Session Memory — earlier context]\n{}", summary))
            {
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
            messages: messages.clone(),
        };
        let response = self.provider.complete(request).await?;
        let mut total_usage = response.usage;

        // Check for tool calls in the response
        let final_content = if let Some(ref tools) = self.tools {
            if let Some((call, surrounding_text)) =
                crate::application::tool_dispatch::parse_tool_call(&response.content)
            {
                tracing::info!(
                    agent = %self.persona.id,
                    tool = call.tool.as_str(),
                    "agent invoked tool"
                );
                let result = crate::application::tool_dispatch::dispatch(tools.as_ref(), &call);
                let result_text = crate::application::tool_dispatch::format_result(&result);

                // Re-prompt with the tool result
                messages.push(
                    Message::assistant(self.persona.id.clone(), response.content)
                        .map_err(|e| LlmError::InvalidResponse(e.to_string()))?,
                );
                if let Ok(tool_msg) = Message::system(result_text) {
                    messages.push(tool_msg);
                }
                let follow_up = CompletionRequest {
                    model: self.model.clone(),
                    messages,
                };
                // Box::pin so recursive await is bounded if we ever do loops, but here we just do 1 follow-up
                let follow_up_response = self.provider.complete(follow_up).await?;
                // Accumulate token usage
                if let (Some(a), Some(b)) = (total_usage, follow_up_response.usage) {
                    total_usage = Some(TokenUsage {
                        prompt_tokens: a.prompt_tokens + b.prompt_tokens,
                        completion_tokens: a.completion_tokens + b.completion_tokens,
                    });
                } else {
                    total_usage = total_usage.or(follow_up_response.usage);
                }
                // Return surrounding text + follow up response
                if surrounding_text.is_empty() {
                    follow_up_response.content
                } else {
                    format!("{}\n{}", surrounding_text, follow_up_response.content)
                }
            } else {
                response.content
            }
        } else {
            response.content
        };

        self.last_usage = total_usage;
        self.last_thinking = None;

        // 1. Check for a directed message.
        if let Some(directed_msg) = parse_directed_send(&final_content, self.id()) {
            return Ok(Some(directed_msg));
        }

        // 2. Otherwise return broadcast.
        let message = Message::assistant(self.persona.id.clone(), final_content)
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

    fn last_usage(&self) -> Option<TokenUsage> {
        self.last_usage
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

/// Check if the LLM output contains a directed message marker.
fn parse_directed_send(output: &str, sender: &AgentId) -> Option<Message> {
    let start = output.find("[SEND_TO ")?;
    let end_marker = output.find("[/SEND_TO]")?;
    if end_marker < start {
        return None;
    }
    let header_end = output[start..].find(']')? + start;
    let recipient_str = output[start + 9..header_end].trim();
    let content = output[header_end + 1..end_marker].trim();
    let recipient = AgentId::new(recipient_str).ok()?;
    Message::directed(sender.clone(), recipient, content).ok()
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
                usage: None,
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
