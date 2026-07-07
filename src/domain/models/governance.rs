//! Governance structure — the mandatory markdown scaffold (TASK 005).
//!
//! Maestro requires a governance folder declaring `scopes`, `personas`, and
//! `skills`. This module holds the pure validation logic; the filesystem scan
//! lives in `crate::application::governance`.

use serde::{Deserialize, Serialize};

/// The governance entries every project must provide.
pub const REQUIRED_GOVERNANCE_ENTRIES: [&str; 3] = ["scopes", "personas", "skills"];

/// Outcome of validating a governance folder against the required entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceReport {
    /// Required entries that were not found.
    pub missing: Vec<String>,
}

impl GovernanceReport {
    /// `true` when nothing is missing.
    pub fn is_valid(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Validate a set of present entry names against [`REQUIRED_GOVERNANCE_ENTRIES`].
///
/// Pure: the caller supplies the names discovered on disk.
pub fn validate_entries(present: &[String]) -> GovernanceReport {
    let missing = REQUIRED_GOVERNANCE_ENTRIES
        .iter()
        .filter(|required| !present.iter().any(|p| p == *required))
        .map(|s| (*s).to_string())
        .collect();
    GovernanceReport { missing }
}

/// A governed entry kind. Config Mode manages all four (ADR 0002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceKind {
    /// `maestro/config.yml`.
    Config,
    /// A persona (instructions) under `maestro/personas/`.
    Persona,
    /// A skill under `maestro/skills/`.
    Skill,
    /// A project scope under `maestro/scopes/`.
    Scope,
}

impl GovernanceKind {
    /// The governance sub-directory for this kind (`Config` lives at the root).
    pub fn dir(self) -> Option<&'static str> {
        match self {
            GovernanceKind::Config => None,
            GovernanceKind::Persona => Some("personas"),
            GovernanceKind::Skill => Some("skills"),
            GovernanceKind::Scope => Some("scopes"),
        }
    }

    /// The snake_case label used on the IPC boundary.
    pub fn as_str(self) -> &'static str {
        match self {
            GovernanceKind::Config => "config",
            GovernanceKind::Persona => "persona",
            GovernanceKind::Skill => "skill",
            GovernanceKind::Scope => "scope",
        }
    }
}

/// Whether an entry ships with Maestro (`Default`) or is user-authored (`Custom`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// Part of the built-in default catalog.
    Default,
    /// Authored by the user.
    Custom,
}

impl Origin {
    /// The snake_case label used on the IPC boundary.
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Default => "default",
            Origin::Custom => "custom",
        }
    }
}

/// A single governed entry surfaced to Config Mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceEntry {
    /// Path-like id, e.g. `personas/software_engineer` or `config.yml`.
    pub id: String,
    /// The entry kind.
    pub kind: GovernanceKind,
    /// Default vs custom origin.
    pub origin: Origin,
    /// Whether the entry has been archived (soft delete).
    pub archived: bool,
}

/// The immutable orchestrator persona id — never editable or archivable.
pub const MAESTRO_PERSONA_ID: &str = "personas/maestro";

/// Turn a display name into a stable, path-safe slug.
pub fn slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Whether an entry id may never be edited or archived (the Maestro persona).
pub fn is_immutable(id: &str) -> bool {
    id == MAESTRO_PERSONA_ID
}

/// The ids of the built-in default personas, e.g. `personas/software_engineer`.
pub fn default_persona_ids() -> Vec<String> {
    super::default_personas()
        .into_iter()
        .map(|p| format!("personas/{}", slug(&p.id.to_string())))
        .collect()
}

/// The `GovernanceKind` implied by an entry id, if recognisable.
pub fn kind_of(id: &str) -> Option<GovernanceKind> {
    if id == "config.yml" {
        Some(GovernanceKind::Config)
    } else if id.starts_with("personas/") {
        Some(GovernanceKind::Persona)
    } else if id.starts_with("skills/") {
        Some(GovernanceKind::Skill)
    } else if id.starts_with("scopes/") {
        Some(GovernanceKind::Scope)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_present_is_valid() {
        let present = vec![
            "scopes".to_string(),
            "personas".to_string(),
            "skills".to_string(),
        ];
        let report = validate_entries(&present);
        assert!(report.is_valid());
        assert!(report.missing.is_empty());
    }

    #[test]
    fn reports_missing_entries() {
        let present = vec!["personas".to_string()];
        let report = validate_entries(&present);
        assert!(!report.is_valid());
        assert_eq!(report.missing, vec!["scopes", "skills"]);
    }

    #[test]
    fn empty_reports_all_missing() {
        let report = validate_entries(&[]);
        assert_eq!(report.missing.len(), 3);
    }

    #[test]
    fn maestro_persona_is_immutable() {
        assert!(is_immutable(MAESTRO_PERSONA_ID));
        assert!(!is_immutable("personas/software_engineer"));
    }

    #[test]
    fn slug_is_path_safe() {
        assert_eq!(slug("Software Engineer"), "software_engineer");
        assert_eq!(slug("  API/Designer  "), "api_designer");
    }

    #[test]
    fn kind_is_inferred_from_id() {
        assert_eq!(kind_of("config.yml"), Some(GovernanceKind::Config));
        assert_eq!(kind_of("personas/x"), Some(GovernanceKind::Persona));
        assert_eq!(kind_of("skills/x"), Some(GovernanceKind::Skill));
        assert_eq!(kind_of("scopes/x"), Some(GovernanceKind::Scope));
        assert_eq!(kind_of("weird"), None);
    }

    #[test]
    fn default_persona_ids_include_maestro_and_engineer() {
        let ids = default_persona_ids();
        assert!(ids.contains(&MAESTRO_PERSONA_ID.to_string()));
        assert!(ids.contains(&"personas/software_engineer".to_string()));
    }
}
