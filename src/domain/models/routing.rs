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

/// Scoring weights per signal source.
pub const WEIGHT_ID: u32 = 1;
pub const WEIGHT_RESPONSIBILITY: u32 = 1;
pub const WEIGHT_KEYWORDS: u32 = 2;
pub const WEIGHT_SKILL_TAGS: u32 = 2;

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
    /// Human-readable routing explanation.
    pub reason: String,
}

/// Lowercased alphanumeric tokens of length > 2.
fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(|w| w.to_string())
        .collect()
}

/// Weighted lexical overlap between demand tokens and persona signals.
fn score(demand_tokens: &[String], persona: &Persona) -> u32 {
    let id_tokens = tokens(&persona.id.to_string());
    let resp_tokens = tokens(&persona.responsibility);
    let kw_tokens: Vec<String> = persona
        .expertise_keywords
        .iter()
        .flat_map(|kw| tokens(kw))
        .collect();
    let tag_tokens: Vec<String> = persona
        .skill_tags
        .iter()
        .flat_map(|tag| tokens(tag))
        .collect();

    let id_hits = count_hits(demand_tokens, &id_tokens);
    let resp_hits = count_hits(demand_tokens, &resp_tokens);
    let kw_hits = count_hits(demand_tokens, &kw_tokens);
    let tag_hits = count_hits(demand_tokens, &tag_tokens);

    id_hits * WEIGHT_ID
        + resp_hits * WEIGHT_RESPONSIBILITY
        + kw_hits * WEIGHT_KEYWORDS
        + tag_hits * WEIGHT_SKILL_TAGS
}

/// Count how many demand tokens appear in the haystack.
fn count_hits(demand: &[String], haystack: &[String]) -> u32 {
    demand
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

    let (selected, used_fallback) = if winners.is_empty() {
        let fallback = ranked
            .iter()
            .find(|m| m.persona == FALLBACK_PERSONA)
            .or_else(|| ranked.first())
            .map(|m| m.persona.clone());
        (fallback.into_iter().collect(), true)
    } else {
        (winners, false)
    };

    let reason = if used_fallback {
        format!(
            "No persona cleared threshold ({}); fell back to {}",
            MIN_SCORE, FALLBACK_PERSONA
        )
    } else {
        let top = &ranked[0];
        let runner_up = ranked.get(1);
        match runner_up {
            Some(ru) => format!(
                "Selected '{}' (score {}) over '{}' (score {}), margin {}",
                top.persona,
                top.score,
                ru.persona,
                ru.score,
                top.score.saturating_sub(ru.score)
            ),
            None => format!(
                "Selected '{}' (score {}, sole candidate)",
                top.persona, top.score
            ),
        }
    };

    if let Some(top) = ranked.first() {
        tracing::info!(
            selected = %top.persona,
            score = top.score,
            runner_up = ranked.get(1).map(|r| r.persona.as_str()).unwrap_or("none"),
            runner_up_score = ranked.get(1).map(|r| r.score).unwrap_or(0),
            margin = top.score.saturating_sub(ranked.get(1).map(|r| r.score).unwrap_or(0)),
            used_fallback,
            "two-towers routing decision"
        );
    }

    Routing {
        ranked,
        selected,
        used_fallback,
        reason,
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

    #[test]
    fn keywords_improve_routing_precision() {
        let personas = default_personas();
        // "deploy" and "container" match DevOps Engineer keywords, even if not in ID/Responsibility.
        let routing = route("deploy the container", &personas);
        assert!(routing.selected.iter().any(|p| p == "DevOps Engineer"));
        assert!(!routing.used_fallback);
    }

    #[test]
    fn weighted_keywords_beat_id_match() {
        let personas = default_personas();
        // "security" is in ID for Security Analyst (weight 1), but "coding" is a skill_tag for Software Engineer (weight 2).
        let routing = route("coding", &personas);
        assert_eq!(routing.selected, vec!["Software Engineer"]);
        let score = routing
            .ranked
            .iter()
            .find(|m| m.persona == "Software Engineer")
            .unwrap()
            .score;
        assert_eq!(score, WEIGHT_SKILL_TAGS);
    }

    #[test]
    fn skill_tags_contribute_to_score() {
        let personas = default_personas();
        // "infra" is a skill tag for DevOps Engineer.
        let routing = route("infra", &personas);
        assert!(routing.selected.iter().any(|p| p == "DevOps Engineer"));
        let score = routing
            .ranked
            .iter()
            .find(|m| m.persona == "DevOps Engineer")
            .unwrap()
            .score;
        // DevOps has "infra" in both keywords (weight 2) and skill tags (weight 2), total 4
        assert_eq!(score, WEIGHT_KEYWORDS + WEIGHT_SKILL_TAGS);
    }

    #[test]
    fn reason_explains_selection() {
        let personas = default_personas();
        let routing = route("improve quality assurance and testing", &personas);
        assert!(routing.reason.contains("Selected 'Quality Assurance'"));
        assert!(routing.reason.contains("score"));
        assert!(routing.reason.contains("margin"));
    }

    #[test]
    fn reason_explains_fallback() {
        let personas = default_personas();
        let routing = route("zzz qqq vvv", &personas);
        assert!(routing.reason.contains("No persona cleared threshold"));
        assert!(routing.reason.contains("fell back to Software Engineer"));
    }
}
