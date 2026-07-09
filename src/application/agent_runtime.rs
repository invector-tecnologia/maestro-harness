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

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinSet;

use crate::application::agent_metrics::AgentMetrics;
use crate::application::agent_observability::RuntimeEvent;
use crate::domain::models::{AgentId, Message, Scratchpad};
use crate::domain::ports::Role;
use crate::infrastructure::bus::BroadcastBus;

/// Lifecycle status of a registered agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Terminated,
}

/// Tracks registered agents and their lifecycle status.
struct AgentEntry {
    status: AgentStatus,
}

/// Orchestrates agent cognitive cycles and narrates them.
#[derive(Clone)]
pub struct AgentRuntime {
    events: BroadcastBus<RuntimeEvent>,
    agent_bus: BroadcastBus<Message>,
    registry: Arc<RwLock<HashMap<AgentId, AgentEntry>>>,
    metrics: Arc<RwLock<AgentMetrics>>,
}

impl AgentRuntime {
    /// Build a runtime that narrates onto `events`.
    pub fn new(events: BroadcastBus<RuntimeEvent>, agent_bus: BroadcastBus<Message>) -> Self {
        Self {
            events,
            agent_bus,
            registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(AgentMetrics::new())),
        }
    }

    /// Register an agent in the runtime.
    pub async fn register(&self, id: AgentId) {
        self.registry.write().await.insert(
            id.clone(),
            AgentEntry {
                status: AgentStatus::Idle,
            },
        );
        emit(
            &self.events,
            RuntimeEvent::AgentLifecycle {
                agent: id,
                status: "Registered".to_string(),
            },
        )
        .await;
    }

    /// Terminate an agent.
    pub async fn terminate(&self, id: &AgentId) {
        if let Some(entry) = self.registry.write().await.get_mut(id) {
            entry.status = AgentStatus::Terminated;
            emit(
                &self.events,
                RuntimeEvent::AgentLifecycle {
                    agent: id.clone(),
                    status: "Terminated".to_string(),
                },
            )
            .await;
        }
    }

    /// Query the status of an agent.
    pub async fn status(&self, id: &AgentId) -> Option<AgentStatus> {
        self.registry.read().await.get(id).map(|e| e.status)
    }

    /// List all non-terminated agents.
    pub async fn active_agents(&self) -> Vec<AgentId> {
        self.registry
            .read()
            .await
            .iter()
            .filter(|(_, e)| e.status != AgentStatus::Terminated)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Snapshot current metrics.
    pub async fn metrics(&self) -> AgentMetrics {
        self.metrics.read().await.clone()
    }

    /// The narration bus (subscribe to render agent activity).
    pub fn events(&self) -> &BroadcastBus<RuntimeEvent> {
        &self.events
    }

    /// Run one cognitive cycle across `agents` given shared `input`, returning
    /// every produced message. Ordering is not guaranteed (agents run concurrently).
    pub async fn run_cycle(
        &self,
        agents: Vec<Box<dyn Role>>,
        input: Vec<Message>,
        scratchpad: Option<Arc<RwLock<Scratchpad>>>,
    ) -> Vec<Message> {
        let mut set: JoinSet<Option<Message>> = JoinSet::new();

        for mut agent in agents {
            let events = self.events.clone();
            let agent_bus = self.agent_bus.clone();
            let metrics = self.metrics.clone();
            let input = input.clone();
            let pad = scratchpad.clone();
            set.spawn(async move {
                let id = agent.id().clone();

                let history = agent_bus.history().await;
                let visible: Vec<Message> = history
                    .into_iter()
                    .filter(|m| m.is_visible_to(&id))
                    .collect();
                let mut enriched_input = visible;

                if let Some(ref p) = pad {
                    let ctx = p.read().await.as_prompt_context();
                    if !ctx.is_empty() {
                        if let Ok(ctx_msg) = Message::system(ctx) {
                            enriched_input.push(ctx_msg);
                        }
                    }
                }

                enriched_input.extend(input);

                emit(&events, RuntimeEvent::AgentObserving { agent: id.clone() }).await;
                agent.observe(&enriched_input);

                emit(&events, RuntimeEvent::AgentThinking { agent: id.clone() }).await;
                let _thinking = agent.think();

                emit(&events, RuntimeEvent::AgentActing { agent: id.clone() }).await;
                let act_start = std::time::Instant::now();
                match agent.act().await {
                    Ok(output) => {
                        let latency = act_start.elapsed();
                        let usage = agent.last_usage();

                        let mut metrics_guard = metrics.write().await;
                        metrics_guard.record_cycle(&id, true, usage, latency);
                        let stats = metrics_guard.stats(&id).unwrap().clone();
                        drop(metrics_guard);

                        // REFLECT phase: self-critique when output was produced
                        if let Some(ref msg) = output {
                            if msg.is_directed() {
                                if let (Some(sender), Some(recipient)) =
                                    (&msg.author, &msg.recipient)
                                {
                                    emit(
                                        &events,
                                        RuntimeEvent::AgentDirectedSend {
                                            sender: sender.clone(),
                                            recipient: recipient.clone(),
                                        },
                                    )
                                    .await;
                                }
                            }

                            agent_bus.publish(msg.clone()).await;
                            emit(&events, RuntimeEvent::AgentPublished { agent: id.clone() }).await;

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
                                agent: id.clone(),
                                produced: output.is_some(),
                            },
                        )
                        .await;

                        emit(
                            &events,
                            RuntimeEvent::AgentMetricsSnapshot {
                                agent: id,
                                cycles: stats.cycles,
                                successes: stats.successes,
                                failures: stats.failures,
                                prompt_tokens: stats.prompt_tokens,
                                completion_tokens: stats.completion_tokens,
                                latency_ms: stats.total_latency.as_millis() as u64,
                            },
                        )
                        .await;

                        output
                    }
                    Err(error) => {
                        let latency = act_start.elapsed();

                        let mut metrics_guard = metrics.write().await;
                        metrics_guard.record_cycle(&id, false, None, latency);
                        let stats = metrics_guard.stats(&id).unwrap().clone();
                        drop(metrics_guard);

                        emit(
                            &events,
                            RuntimeEvent::AgentFailed {
                                agent: id.clone(),
                                error: error.to_string(),
                            },
                        )
                        .await;

                        emit(
                            &events,
                            RuntimeEvent::AgentMetricsSnapshot {
                                agent: id,
                                cycles: stats.cycles,
                                successes: stats.successes,
                                failures: stats.failures,
                                prompt_tokens: stats.prompt_tokens,
                                completion_tokens: stats.completion_tokens,
                                latency_ms: stats.total_latency.as_millis() as u64,
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
                usage: None,
            })
        });
        Arc::new(mock)
    }

    #[tokio::test]
    async fn runs_cycle_and_collects_outputs() {
        let runtime = AgentRuntime::new(BroadcastBus::new(64, 64), BroadcastBus::new(64, 64));
        let mut narration = runtime.events().subscribe();

        let agents = activate_default_agents(ready_provider(), "mistral");
        let expected = agents.len();
        let outputs = runtime
            .run_cycle(agents, vec![Message::user("build a script").unwrap()], None)
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

        let runtime = AgentRuntime::new(BroadcastBus::new(64, 64), BroadcastBus::new(64, 64));
        let outputs = runtime
            .run_cycle(agents, vec![Message::user("x").unwrap()], None)
            .await;

        // Every agent failed, but the cycle completed without panicking.
        assert!(outputs.is_empty());
    }
}
