//! Readiness — the SENSE stage (TASK 010).
//!
//! Probes provider availability before committing work. Depends only on the
//! domain `LlmProvider` port so orchestration stays decoupled from adapters.

use std::sync::Arc;

use crate::domain::ports::{LlmProvider, ProviderStatus};

/// Probe a provider (if one is resolved) for readiness. A missing provider is
/// reported as `Unreachable` rather than blocking or panicking.
pub async fn probe_provider(provider: Option<Arc<dyn LlmProvider>>) -> ProviderStatus {
    match provider {
        Some(provider) => provider.probe().await,
        None => ProviderStatus::Unreachable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::llm_provider::MockLlmProvider;

    #[tokio::test]
    async fn probes_resolved_provider() {
        let mut mock = MockLlmProvider::new();
        mock.expect_probe().returning(|| ProviderStatus::Available);
        let status = probe_provider(Some(Arc::new(mock))).await;
        assert_eq!(status, ProviderStatus::Available);
    }

    #[tokio::test]
    async fn missing_provider_is_unreachable() {
        assert_eq!(probe_provider(None).await, ProviderStatus::Unreachable);
    }
}
