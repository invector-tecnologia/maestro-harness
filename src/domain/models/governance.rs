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
}
