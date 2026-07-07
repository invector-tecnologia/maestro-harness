//! Provider registry and connector factory (TASK 008).
//!
//! Builds concrete [`LlmProvider`] adapters from configuration and resolves them
//! by key. Unknown provider kinds fail fast.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use crate::domain::models::config::{MaestroConfig, ProviderConfig};
use crate::domain::ports::LlmProvider;

use super::ollama::OllamaProvider;
use super::openai::OpenAiProvider;

/// Default per-request provider timeout.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Errors building the registry.
#[derive(Debug, Error)]
pub enum FactoryError {
    /// The provider `kind` has no adapter.
    #[error("unsupported provider kind '{0}'")]
    UnsupportedKind(String),
    /// Building the adapter failed.
    #[error("failed to build provider '{key}': {source}")]
    Build {
        key: String,
        #[source]
        source: crate::domain::ports::LlmError,
    },
}

/// A resolved set of providers keyed by configuration name.
#[derive(Clone)]
pub struct ProviderRegistry {
    providers: BTreeMap<String, Arc<dyn LlmProvider>>,
}

impl ProviderRegistry {
    /// Build every declared provider from `config`.
    pub fn from_config(config: &MaestroConfig) -> Result<Self, FactoryError> {
        let mut providers = BTreeMap::new();
        for (key, provider_config) in &config.providers {
            providers.insert(key.clone(), build_provider(key, provider_config)?);
        }
        Ok(Self { providers })
    }

    /// Resolve a provider by configuration key.
    pub fn resolve(&self, key: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.get(key).cloned()
    }

    /// Resolve the configured default provider.
    pub fn default_provider(&self, config: &MaestroConfig) -> Option<Arc<dyn LlmProvider>> {
        self.resolve(&config.system.default_provider)
    }

    /// Number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

fn build_provider(
    key: &str,
    config: &ProviderConfig,
) -> Result<Arc<dyn LlmProvider>, FactoryError> {
    match config.kind.as_str() {
        "ollama" => {
            let provider =
                OllamaProvider::new(&config.endpoint, DEFAULT_TIMEOUT).map_err(|source| {
                    FactoryError::Build {
                        key: key.to_string(),
                        source,
                    }
                })?;
            Ok(Arc::new(provider))
        }
        "openai" => {
            // Cloud keys come from the environment, never config files.
            let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
            let provider = OpenAiProvider::new(&config.endpoint, api_key, DEFAULT_TIMEOUT)
                .map_err(|source| FactoryError::Build {
                    key: key.to_string(),
                    source,
                })?;
            Ok(Arc::new(provider))
        }
        other => Err(FactoryError::UnsupportedKind(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::config::{ModelConfig, SystemConfig};

    fn config_with_kind(kind: &str) -> MaestroConfig {
        let mut providers = BTreeMap::new();
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                kind: kind.to_string(),
                endpoint: "http://127.0.0.1:11434/v1".to_string(),
                models: vec![ModelConfig {
                    name: "mistral".to_string(),
                }],
            },
        );
        MaestroConfig {
            system: SystemConfig {
                default_provider: "ollama".to_string(),
                default_model: "mistral".to_string(),
                max_concurrency: 4,
            },
            providers,
            agents: BTreeMap::new(),
        }
    }

    #[test]
    fn builds_and_resolves_ollama() {
        let config = config_with_kind("ollama");
        let registry = ProviderRegistry::from_config(&config).unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.resolve("ollama").is_some());
        assert!(registry.default_provider(&config).is_some());
    }

    #[test]
    fn unknown_kind_fails_fast() {
        let config = config_with_kind("mystery");
        let result = ProviderRegistry::from_config(&config);
        assert!(matches!(result, Err(FactoryError::UnsupportedKind(k)) if k == "mystery"));
    }

    #[test]
    fn builds_and_resolves_openai() {
        let mut config = config_with_kind("openai");
        config.providers.get_mut("ollama").unwrap().endpoint =
            "https://api.openai.com/v1".to_string();
        let registry = ProviderRegistry::from_config(&config).unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.resolve("ollama").is_some());
    }
}
