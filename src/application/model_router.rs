//! Model routing — per-persona model resolution (TASK 042).
//!
//! Resolves the model a persona should run on from the config `agents` bindings,
//! falling back to the system default model. Pure over its inputs.

use crate::domain::models::MaestroConfig;

/// Describes how a model was resolved for a persona.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelResolution {
    /// The provider key.
    pub provider: String,
    /// The resolved model name.
    pub model: String,
    /// How it was resolved.
    pub source: ResolutionSource,
    /// The full cascade chain (primary → fallback → default), for narration.
    pub cascade: Vec<String>,
}

/// Why this model was chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionSource {
    /// Persona had an explicit binding in config.
    Bound,
    /// Persona had a binding with a fallback, and fallback was selected.
    Fallback,
    /// No binding; system default was used.
    Default,
}

/// The model bound to `persona`, or the system default when unbound.
pub fn model_for(config: &MaestroConfig, persona: &str) -> ModelResolution {
    if let Some(binding) = config.agents.get(persona) {
        let mut cascade = vec![binding.model.clone()];
        if let Some(ref fallback) = binding.fallback_model {
            cascade.push(fallback.clone());
        }
        cascade.push(config.system.default_model.clone());

        tracing::info!(
            persona = persona,
            model = %binding.model,
            provider = %binding.provider,
            fallback = binding.fallback_model.as_deref().unwrap_or("none"),
            source = "bound",
            "model resolution"
        );

        ModelResolution {
            provider: binding.provider.clone(),
            model: binding.model.clone(),
            source: ResolutionSource::Bound,
            cascade,
        }
    } else {
        tracing::info!(
            persona = persona,
            model = %config.system.default_model,
            provider = %config.system.default_provider,
            source = "default",
            "model resolution"
        );

        ModelResolution {
            provider: config.system.default_provider.clone(),
            model: config.system.default_model.clone(),
            source: ResolutionSource::Default,
            cascade: vec![config.system.default_model.clone()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{AgentBinding, SystemConfig};
    use std::collections::BTreeMap;

    fn config_with(agents: BTreeMap<String, AgentBinding>) -> MaestroConfig {
        MaestroConfig {
            system: SystemConfig {
                default_provider: "ollama".to_string(),
                default_model: "mistral".to_string(),
                max_concurrency: 4,
            },
            providers: BTreeMap::new(),
            agents,
        }
    }

    #[test]
    fn bound_persona_resolves_with_bound_source() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "Software Engineer".to_string(),
            AgentBinding {
                provider: "ollama".to_string(),
                model: "codellama".to_string(),
                fallback_model: None,
            },
        );
        let config = config_with(agents);
        let res = model_for(&config, "Software Engineer");
        assert_eq!(res.model, "codellama");
        assert_eq!(res.source, ResolutionSource::Bound);
        assert_eq!(
            res.cascade,
            vec!["codellama".to_string(), "mistral".to_string()]
        );
    }

    #[test]
    fn fallback_model_appears_in_cascade() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "Software Engineer".to_string(),
            AgentBinding {
                provider: "ollama".to_string(),
                model: "codellama".to_string(),
                fallback_model: Some("gpt-4".to_string()),
            },
        );
        let config = config_with(agents);
        let res = model_for(&config, "Software Engineer");
        assert_eq!(res.model, "codellama");
        assert_eq!(
            res.cascade,
            vec![
                "codellama".to_string(),
                "gpt-4".to_string(),
                "mistral".to_string()
            ]
        );
    }

    #[test]
    fn unbound_persona_resolves_with_default_source() {
        let config = config_with(BTreeMap::new());
        let res = model_for(&config, "Quality Assurance");
        assert_eq!(res.model, "mistral");
        assert_eq!(res.source, ResolutionSource::Default);
        assert_eq!(res.cascade, vec!["mistral".to_string()]);
    }
}
