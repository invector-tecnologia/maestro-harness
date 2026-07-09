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
    /// Multi-line system prompt template. Injected as the first message in act().
    #[serde(default)]
    pub system_prompt: String,
    /// Expertise keywords for Two-Towers routing (enriches scoring signal).
    #[serde(default)]
    pub expertise_keywords: Vec<String>,
    /// Skill tags this persona is associated with (future skill binding).
    #[serde(default)]
    pub skill_tags: Vec<String>,
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
        system_prompt: impl Into<String>,
        expertise_keywords: Vec<String>,
        skill_tags: Vec<String>,
    ) -> Result<Self, PersonaError> {
        let responsibility = responsibility.into();
        let system_prompt = system_prompt.into();
        if responsibility.trim().is_empty() {
            return Err(PersonaError::EmptyResponsibility(id.to_string()));
        }
        Ok(Self {
            id,
            responsibility,
            can_handoff_to,
            orchestrator,
            system_prompt,
            expertise_keywords,
            skill_tags,
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
    let devops = id("DevOps Engineer");
    let sec = id("Security Analyst");
    let tw = id("Technical Writer");

    let all_operational = vec![
        pm.clone(),
        qa.clone(),
        ux.clone(),
        eng.clone(),
        devops.clone(),
        sec.clone(),
        tw.clone(),
    ];

    vec![
        persona(
            maestro.clone(),
            "Orchestrate the micro-project: plan, delegate, audit, deliver.",
            all_operational,
            true,
            "You are Maestro, the master orchestrator. Your job is to plan, delegate, and audit the work of other personas.",
            vec!["orchestrator".to_string(), "manager".to_string()],
            vec![],
        ),
        persona(
            pm,
            "Break the demand into a validated plan and track delivery.",
            vec![maestro.clone()],
            false,
            "You are a Project Manager. Focus on organizing tasks, setting milestones, and ensuring the plan covers all requirements.",
            vec!["planning".to_string(), "management".to_string(), "delivery".to_string()],
            vec!["planning".to_string()],
        ),
        persona(
            qa,
            "Validate contributions against acceptance criteria.",
            vec![maestro.clone()],
            false,
            "You are a QA Engineer. Your focus is on testing, finding edge cases, and verifying that all acceptance criteria are met.",
            vec!["testing".to_string(), "qa".to_string(), "validation".to_string()],
            vec!["testing".to_string()],
        ),
        persona(
            ux,
            "Shape usable, accessible terminal-first experiences.",
            vec![maestro.clone()],
            false,
            "You are a UX Designer. Focus on user flows, clarity, and terminal-first accessibility.",
            vec!["design".to_string(), "ux".to_string(), "accessibility".to_string()],
            vec!["design".to_string()],
        ),
        persona(
            eng,
            "Implement the solution with tests and safe execution.",
            vec![maestro.clone()],
            false,
            "You are a Software Engineer. Write robust, idiomatic code, write tests, and safely execute implementation steps.",
            vec!["code".to_string(), "implementation".to_string(), "engineer".to_string(), "rust".to_string()],
            vec!["coding".to_string()],
        ),
        persona(
            devops,
            "Automate infrastructure, CI/CD, and environment provisioning.",
            vec![maestro.clone()],
            false,
            "You are a DevOps Engineer. Focus on infrastructure as code, CI/CD pipelines, and environment management.",
            vec!["ci".to_string(), "cd".to_string(), "pipeline".to_string(), "docker".to_string(), "container".to_string(), "deploy".to_string(), "infra".to_string(), "terraform".to_string(), "kubernetes".to_string()],
            vec!["infra".to_string(), "ci-cd".to_string()],
        ),
        persona(
            sec,
            "Identify threats, review access controls, and enforce security policy.",
            vec![maestro.clone()],
            false,
            "You are a Security Analyst. Review all code and infra for vulnerabilities, focusing on access controls and safe practices.",
            vec!["security".to_string(), "threat".to_string(), "vulnerability".to_string(), "auth".to_string(), "access".to_string(), "policy".to_string(), "encryption".to_string(), "audit".to_string()],
            vec!["security".to_string()],
        ),
        persona(
            tw,
            "Produce clear documentation, READMEs, and user guides.",
            vec![maestro],
            false,
            "You are a Technical Writer. Your goal is to produce clear, accurate, and helpful documentation for developers and users.",
            vec!["documentation".to_string(), "readme".to_string(), "guide".to_string(), "writing".to_string(), "docs".to_string(), "api-docs".to_string(), "changelog".to_string()],
            vec!["docs".to_string()],
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
    system_prompt: &str,
    expertise_keywords: Vec<String>,
    skill_tags: Vec<String>,
) -> Persona {
    Persona::new(
        id,
        responsibility,
        handoffs,
        orchestrator,
        system_prompt,
        expertise_keywords,
        skill_tags,
    )
    .expect("default persona responsibility is non-empty")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_has_eight_personas() {
        let catalog = default_personas();
        assert_eq!(catalog.len(), 8);
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
        assert_eq!(maestro.can_handoff_to.len(), 7);
    }

    #[test]
    fn rejects_empty_responsibility() {
        let err = Persona::new(
            AgentId::new("X").unwrap(),
            "  ",
            vec![],
            false,
            "",
            vec![],
            vec![],
        )
        .unwrap_err();
        assert_eq!(err, PersonaError::EmptyResponsibility("X".to_string()));
    }

    #[test]
    fn all_default_personas_have_system_prompts() {
        let catalog = default_personas();
        assert!(catalog.iter().all(|p| !p.system_prompt.is_empty()));
    }

    #[test]
    fn all_operational_personas_have_expertise_keywords() {
        let catalog = default_personas();
        assert!(catalog
            .iter()
            .filter(|p| !p.orchestrator)
            .all(|p| !p.expertise_keywords.is_empty()));
    }
}
