//! Configuration schema (TASK 006) — pure types and validation.
//!
//! Loading (XDG fallback, legacy `config.yaml`) lives in
//! `crate::infrastructure::config`. This module only models and validates.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Top-level system defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Provider used when a persona does not pin one.
    pub default_provider: String,
    /// Model used when a persona does not pin one.
    pub default_model: String,
    /// Maximum number of concurrently running agents.
    #[serde(default = "default_concurrency")]
    pub max_concurrency: u32,
}

fn default_concurrency() -> u32 {
    4
}

/// A configured model within a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model name (e.g. `mistral`).
    pub name: String,
}

/// A configured provider and its models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider kind (e.g. `ollama`, `openai`).
    pub kind: String,
    /// Endpoint URL.
    pub endpoint: String,
    /// Declared models.
    #[serde(default)]
    pub models: Vec<ModelConfig>,
}

/// A per-persona provider+model pinning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentBinding {
    /// Provider key (must exist in `providers`).
    pub provider: String,
    /// Model name (must exist under that provider).
    pub model: String,
}

/// The full Maestro configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaestroConfig {
    /// System defaults.
    pub system: SystemConfig,
    /// Providers by key.
    pub providers: BTreeMap<String, ProviderConfig>,
    /// Optional per-persona bindings.
    #[serde(default)]
    pub agents: BTreeMap<String, AgentBinding>,
}

/// Configuration validation failures (fail fast, per conventions).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// The default provider is not declared under `providers`.
    #[error("default provider '{0}' is not declared under providers")]
    UnknownDefaultProvider(String),
    /// The default model is not declared under the default provider.
    #[error("default model '{model}' is not declared under provider '{provider}'")]
    UnknownDefaultModel { provider: String, model: String },
    /// A persona binding references an undeclared provider.
    #[error("agent '{agent}' references unknown provider '{provider}'")]
    UnknownAgentProvider { agent: String, provider: String },
    /// A persona binding references an undeclared model.
    #[error("agent '{agent}' references unknown model '{model}' on provider '{provider}'")]
    UnknownAgentModel {
        agent: String,
        provider: String,
        model: String,
    },
}

impl MaestroConfig {
    /// Validate cross-references: every referenced provider/model must exist.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let default = self.providers.get(&self.system.default_provider).ok_or(
            ConfigError::UnknownDefaultProvider(self.system.default_provider.clone()),
        )?;
        if !model_exists(default, &self.system.default_model) {
            return Err(ConfigError::UnknownDefaultModel {
                provider: self.system.default_provider.clone(),
                model: self.system.default_model.clone(),
            });
        }

        for (agent, binding) in &self.agents {
            let provider = self.providers.get(&binding.provider).ok_or_else(|| {
                ConfigError::UnknownAgentProvider {
                    agent: agent.clone(),
                    provider: binding.provider.clone(),
                }
            })?;
            if !model_exists(provider, &binding.model) {
                return Err(ConfigError::UnknownAgentModel {
                    agent: agent.clone(),
                    provider: binding.provider.clone(),
                    model: binding.model.clone(),
                });
            }
        }
        Ok(())
    }
}

fn model_exists(provider: &ProviderConfig, model: &str) -> bool {
    provider.models.iter().any(|m| m.name == model)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_config() -> MaestroConfig {
        let mut providers = BTreeMap::new();
        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                kind: "ollama".to_string(),
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
    fn valid_config_passes() {
        assert!(base_config().validate().is_ok());
    }

    #[test]
    fn unknown_default_provider_fails_fast() {
        let mut cfg = base_config();
        cfg.system.default_provider = "openai".to_string();
        assert_eq!(
            cfg.validate(),
            Err(ConfigError::UnknownDefaultProvider("openai".to_string()))
        );
    }

    #[test]
    fn unknown_agent_model_fails_fast() {
        let mut cfg = base_config();
        cfg.agents.insert(
            "Maestro".to_string(),
            AgentBinding {
                provider: "ollama".to_string(),
                model: "gpt-4".to_string(),
            },
        );
        assert_eq!(
            cfg.validate(),
            Err(ConfigError::UnknownAgentModel {
                agent: "Maestro".to_string(),
                provider: "ollama".to_string(),
                model: "gpt-4".to_string(),
            })
        );
    }

    #[test]
    fn yaml_round_trips() {
        let cfg = base_config();
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let back: MaestroConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(cfg, back);
    }
}
