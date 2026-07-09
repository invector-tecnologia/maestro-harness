//! Per-agent metrics aggregation for observability.

use crate::domain::models::AgentId;
use crate::domain::ports::TokenUsage;
use std::collections::HashMap;
use std::time::Duration;

/// Accumulated metrics for a single agent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentStats {
    /// Total cognitive cycles run.
    pub cycles: u64,
    /// Successful completions (act produced a message).
    pub successes: u64,
    /// Failed completions (act returned an error).
    pub failures: u64,
    /// Total prompt tokens consumed.
    pub prompt_tokens: u64,
    /// Total completion tokens generated.
    pub completion_tokens: u64,
    /// Total time spent in act() calls.
    pub total_latency: Duration,
}

impl AgentStats {
    /// Total tokens (prompt + completion).
    pub fn total_tokens(&self) -> u64 {
        self.prompt_tokens + self.completion_tokens
    }

    /// Success rate as a fraction [0.0, 1.0].
    pub fn success_rate(&self) -> f64 {
        if self.cycles == 0 {
            return 0.0;
        }
        self.successes as f64 / self.cycles as f64
    }
}

/// Aggregates metrics across all agents in a session.
#[derive(Debug, Clone, Default)]
pub struct AgentMetrics {
    stats: HashMap<AgentId, AgentStats>,
}

impl AgentMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a completed cycle for an agent.
    pub fn record_cycle(
        &mut self,
        agent: &AgentId,
        success: bool,
        usage: Option<TokenUsage>,
        latency: Duration,
    ) {
        let entry = self.stats.entry(agent.clone()).or_default();
        entry.cycles += 1;
        if success {
            entry.successes += 1;
        } else {
            entry.failures += 1;
        }
        if let Some(u) = usage {
            entry.prompt_tokens += u.prompt_tokens;
            entry.completion_tokens += u.completion_tokens;
        }
        entry.total_latency += latency;
    }

    /// Get stats for a specific agent.
    pub fn stats(&self, agent: &AgentId) -> Option<&AgentStats> {
        self.stats.get(agent)
    }

    /// Session-wide totals.
    pub fn session_totals(&self) -> AgentStats {
        let mut totals = AgentStats::default();
        for stats in self.stats.values() {
            totals.cycles += stats.cycles;
            totals.successes += stats.successes;
            totals.failures += stats.failures;
            totals.prompt_tokens += stats.prompt_tokens;
            totals.completion_tokens += stats.completion_tokens;
            totals.total_latency += stats.total_latency;
        }
        totals
    }

    /// All per-agent stats, sorted by agent ID for determinism.
    pub fn all_stats(&self) -> Vec<(&AgentId, &AgentStats)> {
        let mut items: Vec<_> = self.stats.iter().collect();
        items.sort_by_key(|(id, _)| (*id).clone());
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_cycle_increments_counts() {
        let mut metrics = AgentMetrics::new();
        let agent = AgentId::new("test").unwrap();
        metrics.record_cycle(&agent, true, None, Duration::from_millis(100));
        metrics.record_cycle(&agent, false, None, Duration::from_millis(50));

        let stats = metrics.stats(&agent).unwrap();
        assert_eq!(stats.cycles, 2);
        assert_eq!(stats.successes, 1);
        assert_eq!(stats.failures, 1);
        assert_eq!(stats.total_latency, Duration::from_millis(150));
    }

    #[test]
    fn session_totals_aggregate_all_agents() {
        let mut metrics = AgentMetrics::new();
        let agent1 = AgentId::new("a1").unwrap();
        let agent2 = AgentId::new("a2").unwrap();

        metrics.record_cycle(
            &agent1,
            true,
            Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
            }),
            Duration::from_millis(100),
        );
        metrics.record_cycle(
            &agent2,
            true,
            Some(TokenUsage {
                prompt_tokens: 20,
                completion_tokens: 10,
            }),
            Duration::from_millis(200),
        );

        let totals = metrics.session_totals();
        assert_eq!(totals.cycles, 2);
        assert_eq!(totals.successes, 2);
        assert_eq!(totals.prompt_tokens, 30);
        assert_eq!(totals.completion_tokens, 15);
        assert_eq!(totals.total_tokens(), 45);
        assert_eq!(totals.total_latency, Duration::from_millis(300));
    }

    #[test]
    fn success_rate_is_correct() {
        let mut metrics = AgentMetrics::new();
        let agent = AgentId::new("test").unwrap();

        assert_eq!(metrics.session_totals().success_rate(), 0.0);

        metrics.record_cycle(&agent, true, None, Duration::default());
        metrics.record_cycle(&agent, false, None, Duration::default());

        assert_eq!(metrics.stats(&agent).unwrap().success_rate(), 0.5);
    }

    #[test]
    fn usage_accumulates_tokens() {
        let mut metrics = AgentMetrics::new();
        let agent = AgentId::new("test").unwrap();

        metrics.record_cycle(
            &agent,
            true,
            Some(TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 2,
            }),
            Duration::default(),
        );
        metrics.record_cycle(&agent, true, None, Duration::default());
        metrics.record_cycle(
            &agent,
            true,
            Some(TokenUsage {
                prompt_tokens: 3,
                completion_tokens: 1,
            }),
            Duration::default(),
        );

        let stats = metrics.stats(&agent).unwrap();
        assert_eq!(stats.prompt_tokens, 8);
        assert_eq!(stats.completion_tokens, 3);
        assert_eq!(stats.total_tokens(), 11);
    }
}
