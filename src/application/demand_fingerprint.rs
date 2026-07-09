//! Demand fingerprinting for session transcript matching.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Produce a hex fingerprint for a demand string.
/// Uses a fast, deterministic hash — not cryptographic, but sufficient
/// for local session matching.
pub fn fingerprint(demand: &str) -> String {
    let normalized = demand.trim().to_lowercase();
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_demand_same_fingerprint() {
        assert_eq!(fingerprint("build a cli"), fingerprint("build a cli"));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(fingerprint("Build a CLI"), fingerprint("build a cli"));
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(fingerprint("  build a cli  "), fingerprint("build a cli"));
    }

    #[test]
    fn different_demands_differ() {
        assert_ne!(fingerprint("build a cli"), fingerprint("build a web app"));
    }
}
