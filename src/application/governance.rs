//! Governance folder scanning (TASK 005).
//!
//! Lists the entries under a governance directory and validates them against the
//! required set using the pure `crate::domain::models::governance` logic.

use std::path::Path;

use crate::domain::models::governance::{validate_entries, GovernanceReport};

/// Scan `governance_dir` and validate that `scopes`, `personas`, and `skills`
/// are present. A missing directory yields a report listing everything as missing.
pub fn validate_dir(governance_dir: &Path) -> std::io::Result<GovernanceReport> {
    let mut present = Vec::new();
    if governance_dir.is_dir() {
        for entry in std::fs::read_dir(governance_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                present.push(name.to_string());
            }
        }
    }
    Ok(validate_entries(&present))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_when_all_entries_present() {
        let dir = std::env::temp_dir().join(format!("maestro-gov-{}", std::process::id()));
        for entry in ["scopes", "personas", "skills"] {
            std::fs::create_dir_all(dir.join(entry)).unwrap();
        }
        let report = validate_dir(&dir).unwrap();
        assert!(report.is_valid());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_dir_reports_all_missing() {
        let dir =
            std::env::temp_dir().join(format!("maestro-gov-absent-{}", std::process::id() + 1));
        let report = validate_dir(&dir).unwrap();
        assert_eq!(report.missing.len(), 3);
    }
}
