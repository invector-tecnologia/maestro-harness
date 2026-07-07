//! Two-Towers persona↔task routing (TASK 047).
//!
//! A deterministic matcher: a lexical scorer stands in for the persona/skill
//! embedding towers until an embeddings port lands in `infrastructure`. It scores
//! every candidate persona against the demand, ranks them stably (score desc, then
//! id asc), and applies a documented fallback when nothing clears the threshold.
//! Pure and reproducible — identical input yields identical ranking.

use serde::{Deserialize, Serialize};

use super::persona::Persona;

/// Minimum lexical overlap for a confident match.
pub const MIN_SCORE: u32 = 1;

/// The default persona selected when nothing clears [`MIN_SCORE`].
pub const FALLBACK_PERSONA: &str = "Software Engineer";

/// A scored persona candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonaMatch {
    /// The persona id.
    pub persona: String,
    /// The lexical overlap score.
    pub score: u32,
}

/// A routing decision: the full ranking plus the selected personas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Routing {
    /// All candidates, ranked (score desc, id asc).
    pub ranked: Vec<PersonaMatch>,
    /// The personas chosen to act (threshold winners, or the fallback).
    pub selected: Vec<String>,
    /// Whether the fallback was applied because nothing cleared the threshold.
    pub used_fallback: bool,
}

/// Lowercased alphanumeric tokens of length > 2.
fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.to_string())
        .collect()
}

/// Lexical overlap between the demand tokens and a persona's id + responsibility.
fn score(demand_tokens: &[String], persona: &Persona) -> u32 {
    let mut haystack = tokens(&persona.id.to_string());
    haystack.extend(tokens(&persona.responsibility));
    demand_tokens
        .iter()
        .filter(|t| haystack.iter().any(|h| h == *t))
        .count() as u32
}

/// Route a demand to personas. Excludes the orchestrator; deterministic ranking.
pub fn route(demand: &str, personas: &[Persona]) -> Routing {
    let demand_tokens = tokens(demand);

    let mut ranked: Vec<PersonaMatch> = personas
        .iter()
        .filter(|p| !p.orchestrator)
        .map(|p| PersonaMatch {
            persona: p.id.to_string(),
            score: score(&demand_tokens, p),
        })
        .collect();
    // Stable order: score descending, then id ascending as the tie-breaker.
    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.persona.cmp(&b.persona))
    });

    let winners: Vec<String> = ranked
        .iter()
        .filter(|m| m.score >= MIN_SCORE)
        .map(|m| m.persona.clone())
        .collect();

    if winners.is_empty() {
        let fallback = ranked
            .iter()
            .find(|m| m.persona == FALLBACK_PERSONA)
            .or_else(|| ranked.first())
            .map(|m| m.persona.clone());
        Routing {
            ranked,
            selected: fallback.into_iter().collect(),
            used_fallback: true,
        }
    } else {
        Routing {
            ranked,
            selected: winners,
            used_fallback: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::default_personas;

    #[test]
    fn ranking_is_deterministic() {
        let personas = default_personas();
        let a = route("design the user experience layout", &personas);
        let b = route("design the user experience layout", &personas);
        assert_eq!(a.ranked, b.ranked);
        assert_eq!(a.selected, b.selected);
    }

    #[test]
    fn excludes_the_orchestrator() {
        let personas = default_personas();
        let routing = route("anything at all here", &personas);
        assert!(routing.ranked.iter().all(|m| m.persona != "Maestro"));
    }

    #[test]
    fn matches_relevant_persona_over_others() {
        let personas = default_personas();
        // "quality" and "assurance" overlap the QA persona id/responsibility.
        let routing = route("improve quality assurance and testing", &personas);
        assert!(routing.selected.iter().any(|p| p == "Quality Assurance"));
        assert!(!routing.used_fallback);
    }

    #[test]
    fn falls_back_when_nothing_matches() {
        let personas = default_personas();
        let routing = route("zzz qqq vvv", &personas);
        assert!(routing.used_fallback);
        assert_eq!(routing.selected, vec![FALLBACK_PERSONA.to_string()]);
    }

    #[test]
    fn ties_break_by_id_ascending() {
        let personas = default_personas();
        // A demand that matches nobody gives every candidate score 0 (a tie);
        // the ranking must still be stable and id-ascending.
        let routing = route("0000", &personas);
        let ids: Vec<&str> = routing.ranked.iter().map(|m| m.persona.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }
}
