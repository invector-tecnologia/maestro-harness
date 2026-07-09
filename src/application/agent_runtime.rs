//! Multi-agent runtime (TASK 010).
//!
//! Runs the `OBSERVE → THINK → ACT` cycle for a set of agents concurrently,
//! narrating each phase through the event bus. A failing agent is isolated:
//! its error is narrated and it contributes nothing, but the cycle continues.
//!
//! ## Concurrency vs. the serial cascade
//! Cognitive cycles here run concurrently (`JoinSet`) because they are
//! **read-only reasoning**: `observe`/`think` mutate only per-agent working memory
//! and `act` performs provider calls that do not touch the environment. This does
//! **not** violate the serial-cascade rule. That rule forbids parallel
//! **environment-affecting** steps and is enforced separately by the cascade
//! executor (TASK 048), which runs strictly serially with rollback + approval gates.

use tokio::task::JoinSet;

use crate::application::agent_observability::RuntimeEvent;
use crate::domain::models::Message;
use crate::domain::ports::Role;
use crate::infrastructure::bus::BroadcastBus;

/// Orchestrates agent cognitive cycles and narrates them.
#[derive(Clone)]
pub struct AgentRuntime {
    events: BroadcastBus<RuntimeEvent>,
}

impl AgentRuntime {
    /// Build a runtime that narrates onto `events`.
    pub fn new(events: BroadcastBus<RuntimeEvent>) -> Self {
        Self { events }
    }

    /// The narration bus (subscribe to render agent activity).
    pub fn events(&self) -> &BroadcastBus<RuntimeEvent> {
        &self.events
    }

    /// Run one cognitive cycle across `agents` given shared `input`, returning
    /// every produced message. Ordering is not guaranteed (agents run concurrently).
    pub async fn run_cycle(&self, agents: Vec<Box<dyn Role>>, input: Vec<Message>) -> Vec<Message> {
        let mut set: JoinSet<Option<Message>> = JoinSet::new();

        for mut agent in agents {
            let events = self.events.clone();
            let input = input.clone();
            set.spawn(async move {
                let id = agent.id().clone();

                emit(&events, RuntimeEvent::AgentObserving { agent: id.clone() }).await;
                agent.observe(&input);

                emit(&events, RuntimeEvent::AgentThinking { agent: id.clone() }).await;
                let _thinking = agent.think();

                emit(&events, RuntimeEvent::AgentActing { agent: id.clone() }).await;
                match agent.act().await {
                    Ok(output) => {
                        // REFLECT phase: self-critique when output was produced
                        if let Some(ref msg) = output {
                            let reflection = agent.reflect(msg);
                            emit(
                                &events,
                                RuntimeEvent::AgentReflected {
                                    agent: id.clone(),
                                    satisfied: reflection.satisfied,
                                },
                            )
                            .await;
                        }
                        emit(
                            &events,
                            RuntimeEvent::AgentActed {
                                agent: id,
                                produced: output.is_some(),
                            },
                        )
                        .await;
                        output
                    }
                    Err(error) => {
                        emit(
                            &events,
                            RuntimeEvent::AgentFailed {
                                agent: id,
                                error: error.to_string(),
                            },
                        )
                        .await;
                        None
                    }
                }
            });
        }

        let mut outputs = Vec::new();
        while let Some(joined) = set.join_next().await {
            match joined {
                Ok(Some(message)) => outputs.push(message),
                Ok(None) => {}
                // A task panic is isolated: it does not abort the whole cycle.
                Err(_join_error) => {}
            }
        }
        outputs
    }
}

async fn emit(events: &BroadcastBus<RuntimeEvent>, event: RuntimeEvent) {
    event.narrate();
    events.publish(event).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::persona_agent::activate_default_agents;
    use crate::domain::ports::llm_provider::MockLlmProvider;
    use crate::domain::ports::{CompletionResponse, ProviderStatus};
    use std::sync::Arc;

    fn ready_provider() -> Arc<dyn crate::domain::ports::LlmProvider> {
        let mut mock = MockLlmProvider::new();
        mock.expect_probe().returning(|| ProviderStatus::Available);
        mock.expect_complete().returning(|_req| {
            Ok(CompletionResponse {
                content: "done".to_string(),
            })
        });
        Arc::new(mock)
    }

    #[tokio::test]
    async fn runs_cycle_and_collects_outputs() {
        let runtime = AgentRuntime::new(BroadcastBus::new(64, 64));
        let mut narration = runtime.events().subscribe();

        let agents = activate_default_agents(ready_provider(), "mistral");
        let expected = agents.len();
        let outputs = runtime
            .run_cycle(agents, vec![Message::user("build a script").unwrap()])
            .await;

        assert_eq!(outputs.len(), expected);
        assert!(outputs.iter().all(|m| m.content == "done"));

        // At least one narration event was published.
        assert!(narration.try_recv().is_ok());
    }

    #[tokio::test]
    async fn failing_agent_is_isolated() {
        let mut failing = MockLlmProvider::new();
        failing
            .expect_probe()
            .returning(|| ProviderStatus::Available);
        failing
            .expect_complete()
            .returning(|_r| Err(crate::domain::ports::LlmError::Unauthorized));
        let agents = activate_default_agents(Arc::new(failing), "mistral");

        let runtime = AgentRuntime::new(BroadcastBus::new(64, 64));
        let outputs = runtime
            .run_cycle(agents, vec![Message::user("x").unwrap()])
            .await;

        // Every agent failed, but the cycle completed without panicking.
        assert!(outputs.is_empty());
    }
}
