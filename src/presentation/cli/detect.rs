//! Project auto-detection for `maestro init`.
//!
//! Scans the target directory for well-known project markers (Cargo.toml,
//! package.json, etc.) to infer project context and suggest defaults.

/// A detected project ecosystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedProject {
    /// Human-readable ecosystem name (e.g., "Rust (Cargo)").
    pub ecosystem: String,
    /// Suggested project template key (e.g., "cli-tool", "library").
    pub suggested_template: Option<String>,
    /// The marker file that triggered detection.
    pub marker: String,
}

/// Known project markers, ordered by specificity (most specific first).
const MARKERS: &[(&str, &str, Option<&str>)] = &[
    ("Cargo.toml", "Rust (Cargo)", Some("library")),
    ("package.json", "Node.js (npm)", Some("web-app")),
    ("go.mod", "Go", Some("cli-tool")),
    ("pyproject.toml", "Python (pyproject)", Some("cli-tool")),
    ("setup.py", "Python (setup.py)", Some("cli-tool")),
    ("Makefile", "Make-based project", None),
    (".git", "Git repository", None),
];

/// Scan `dir` for known project markers.
///
/// Returns the first (most specific) match, or `None` if the directory
/// appears to be empty / unrecognised.
pub fn detect(dir: &std::path::Path) -> Option<DetectedProject> {
    for &(marker, ecosystem, template) in MARKERS {
        if dir.join(marker).exists() {
            return Some(DetectedProject {
                ecosystem: ecosystem.to_string(),
                suggested_template: template.map(String::from),
                marker: marker.to_string(),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rust_project() {
        let dir = std::env::temp_dir().join(format!("maestro-detect-rust-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        let result = detect(&dir);
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.ecosystem, "Rust (Cargo)");
        assert_eq!(d.suggested_template.as_deref(), Some("library"));
        assert_eq!(d.marker, "Cargo.toml");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detects_node_project() {
        let dir = std::env::temp_dir().join(format!("maestro-detect-node-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("package.json"), "{}").unwrap();
        let result = detect(&dir);
        assert!(result.is_some());
        assert_eq!(result.unwrap().ecosystem, "Node.js (npm)");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn returns_none_for_empty_dir() {
        let dir = std::env::temp_dir().join(format!("maestro-detect-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(detect(&dir).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn most_specific_marker_wins() {
        // If both Cargo.toml and .git exist, Cargo.toml (more specific) wins.
        let dir = std::env::temp_dir().join(format!("maestro-detect-multi-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "").unwrap();
        let d = detect(&dir).unwrap();
        assert_eq!(d.marker, "Cargo.toml");
        std::fs::remove_dir_all(&dir).ok();
    }
}
