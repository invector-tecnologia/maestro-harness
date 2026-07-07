//! Model routing — per-persona model resolution (TASK 042).
//!
//! Resolves the model a persona should run on from the config `agents` bindings,
//! falling back to the system default model. Pure over its inputs.

use crate::domain::models::MaestroConfig;

/// The model bound to `persona`, or the system default when unbound.
pub fn model_for(config: &MaestroConfig, persona: &str) -> String {
    config
        .agents
        .get(persona)
        .map(|binding| binding.model.clone())
        .unwrap_or_else(|| config.system.default_model.clone())
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
    fn bound_persona_uses_its_model() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "Software Engineer".to_string(),
            AgentBinding {
                provider: "ollama".to_string(),
                model: "codellama".to_string(),
            },
        );
        let config = config_with(agents);
        assert_eq!(model_for(&config, "Software Engineer"), "codellama");
    }

    #[test]
    fn unbound_persona_uses_default_model() {
        let config = config_with(BTreeMap::new());
        assert_eq!(model_for(&config, "Quality Assurance"), "mistral");
    }
}
