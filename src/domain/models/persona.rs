//! Persona model and default catalog (TASK 007).
//!
//! A `Persona` is an AI profile with a single responsibility and an interaction
//! matrix (`can_handoff_to`). The default catalog matches the shipped roster:
//! Maestro (orchestrator) plus four operational personas.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::agent_id::AgentId;

/// An AI profile participating in orchestration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Persona {
    /// Stable identifier (also the persona's display name).
    pub id: AgentId,
    /// One-line responsibility statement.
    pub responsibility: String,
    /// Personas this one may hand work to (the interaction matrix).
    pub can_handoff_to: Vec<AgentId>,
    /// The orchestrator persona is immutable and cannot be deleted.
    #[serde(default)]
    pub orchestrator: bool,
}

/// Errors constructing or validating a [`Persona`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PersonaError {
    /// The responsibility statement was empty.
    #[error("persona '{0}' must declare a responsibility")]
    EmptyResponsibility(String),
}

impl Persona {
    /// Construct and validate a persona.
    pub fn new(
        id: AgentId,
        responsibility: impl Into<String>,
        can_handoff_to: Vec<AgentId>,
        orchestrator: bool,
    ) -> Result<Self, PersonaError> {
        let responsibility = responsibility.into();
        if responsibility.trim().is_empty() {
            return Err(PersonaError::EmptyResponsibility(id.to_string()));
        }
        Ok(Self {
            id,
            responsibility,
            can_handoff_to,
            orchestrator,
        })
    }
}

/// The default persona catalog: `Maestro` (orchestrator) plus the four
/// operational personas. Panics are impossible here — all inputs are valid.
pub fn default_personas() -> Vec<Persona> {
    let maestro = id("Maestro");
    let pm = id("Project Manager");
    let qa = id("Quality Assurance");
    let ux = id("User Experience");
    let eng = id("Software Engineer");

    vec![
        persona(
            maestro.clone(),
            "Orchestrate the micro-project: plan, delegate, audit, deliver.",
            vec![pm.clone(), qa.clone(), ux.clone(), eng.clone()],
            true,
        ),
        persona(
            pm,
            "Break the demand into a validated plan and track delivery.",
            vec![maestro.clone()],
            false,
        ),
        persona(
            qa,
            "Validate contributions against acceptance criteria.",
            vec![maestro.clone()],
            false,
        ),
        persona(
            ux,
            "Shape usable, accessible terminal-first experiences.",
            vec![maestro.clone()],
            false,
        ),
        persona(
            eng,
            "Implement the solution with tests and safe execution.",
            vec![maestro],
            false,
        ),
    ]
}

fn id(name: &str) -> AgentId {
    // The default catalog uses only non-empty names, so construction cannot fail.
    AgentId::new(name).expect("default persona name is non-empty")
}

fn persona(
    id: AgentId,
    responsibility: &str,
    handoffs: Vec<AgentId>,
    orchestrator: bool,
) -> Persona {
    Persona::new(id, responsibility, handoffs, orchestrator)
        .expect("default persona responsibility is non-empty")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_has_five_personas() {
        let catalog = default_personas();
        assert_eq!(catalog.len(), 5);
    }

    #[test]
    fn exactly_one_orchestrator() {
        let orchestrators = default_personas().iter().filter(|p| p.orchestrator).count();
        assert_eq!(orchestrators, 1);
    }

    #[test]
    fn maestro_can_reach_every_operational_persona() {
        let catalog = default_personas();
        let maestro = catalog.iter().find(|p| p.orchestrator).unwrap();
        assert_eq!(maestro.can_handoff_to.len(), 4);
    }

    #[test]
    fn rejects_empty_responsibility() {
        let err = Persona::new(AgentId::new("X").unwrap(), "  ", vec![], false).unwrap_err();
        assert_eq!(err, PersonaError::EmptyResponsibility("X".to_string()));
    }
}
